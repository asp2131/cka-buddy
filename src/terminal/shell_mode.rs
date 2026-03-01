use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::terminal::pty::PtySession;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellMode {
    Embedded,
    External,
}

impl ShellMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Embedded => "embedded",
            Self::External => "external",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "embedded" => Some(Self::Embedded),
            "external" => Some(Self::External),
            _ => None,
        }
    }
}

pub struct ShellRouter {
    mode: ShellMode,
    embedded: PtySession,
    terminal_override: Option<String>,
    external: Option<ExternalBridge>,
    external_output: Vec<String>,
}

impl ShellRouter {
    pub fn new(mode: ShellMode, embedded: PtySession, terminal_override: Option<String>) -> Self {
        let external = if mode == ShellMode::External {
            ExternalBridge::new(terminal_override.clone()).ok()
        } else {
            None
        };

        let mut external_output = Vec::new();
        if mode == ShellMode::External && external.is_none() {
            external_output.push(
                "external mode unavailable: could not initialize bridge; switch to /shell embedded"
                    .to_string(),
            );
        }

        Self {
            mode,
            embedded,
            terminal_override,
            external,
            external_output,
        }
    }

    pub fn mode(&self) -> ShellMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: ShellMode) {
        self.mode = mode;
        if self.mode == ShellMode::External && self.external.is_none() {
            self.external = ExternalBridge::new(self.terminal_override.clone()).ok();
            if self.external.is_none() {
                self.external_output.push(
                    "failed to initialize external bridge; staying unpaired (use /shell embedded)"
                        .to_string(),
                );
            }
        }
    }

    pub fn external_connect_command(&self) -> Option<String> {
        self.external.as_ref().map(ExternalBridge::connect_command)
    }

    pub fn send_line(&mut self, line: &str) -> Result<String> {
        match self.mode {
            ShellMode::Embedded => {
                self.embedded.send_line(line)?;
                Ok(format!("embedded dispatch: {line}"))
            }
            ShellMode::External => {
                let Some(bridge) = self.external.as_mut() else {
                    bail!("external bridge not available; use /shell embedded");
                };

                bridge.poll_accept(&mut self.external_output);

                if !bridge.is_connected() {
                    let msg = format!(
                        "external terminal not connected yet. Run: {}",
                        bridge.connect_command()
                    );
                    self.external_output.push(msg.clone());
                    return Ok(msg);
                }

                bridge.send_line(line).with_context(|| {
                    format!("failed to dispatch command to external bridge: {line}")
                })?;

                let msg = format!("external dispatch: {line}");
                self.external_output.push(msg.clone());
                Ok(msg)
            }
        }
    }

    pub fn drain_output(&mut self) -> Vec<String> {
        match self.mode {
            ShellMode::Embedded => self.embedded.drain_output(),
            ShellMode::External => {
                if let Some(bridge) = self.external.as_mut() {
                    bridge.poll_accept(&mut self.external_output);
                }
                let mut out = Vec::new();
                std::mem::swap(&mut out, &mut self.external_output);
                out
            }
        }
    }
}

struct ExternalBridge {
    listener: TcpListener,
    stream: Option<TcpStream>,
    pin: String,
    terminal: String,
}

impl ExternalBridge {
    fn new(terminal_override: Option<String>) -> Result<Self> {
        let terminal = resolve_terminal_executable(terminal_override)?;
        let listener = TcpListener::bind(("127.0.0.1", 0))?;
        listener.set_nonblocking(true)?;

        Ok(Self {
            listener,
            stream: None,
            pin: generate_pin(),
            terminal,
        })
    }

    fn connect_command(&self) -> String {
        let port = self.listener.local_addr().map(|a| a.port()).unwrap_or(0);
        format!(
            "cargo run -- --shell-connect {} --port {}  # open in {}",
            self.pin, port, self.terminal
        )
    }

    fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    fn poll_accept(&mut self, output: &mut Vec<String>) {
        if self.stream.is_some() {
            return;
        }

        let Ok((stream, addr)) = self.listener.accept() else {
            return;
        };

        if let Ok(cloned) = stream.try_clone() {
            let mut reader = BufReader::new(cloned);
            let mut auth_line = String::new();
            if reader.read_line(&mut auth_line).is_ok() {
                let expected = format!("PIN {}", self.pin);
                if auth_line.trim() == expected {
                    let _ = stream.set_nonblocking(true);
                    self.stream = Some(stream);
                    output.push(format!("external terminal connected from {addr}"));
                    return;
                }
            }
        }

        output.push("external terminal connection rejected (pin mismatch)".to_string());
    }

    fn send_line(&mut self, line: &str) -> Result<()> {
        let Some(stream) = self.stream.as_mut() else {
            bail!("external bridge not connected")
        };

        if let Err(err) = writeln!(stream, "{line}") {
            self.stream = None;
            bail!("bridge disconnected: {err}");
        }
        stream.flush()?;
        Ok(())
    }
}

pub fn run_shell_connector(pin: &str, port: u16) -> Result<()> {
    let mut stream = TcpStream::connect(("127.0.0.1", port))
        .with_context(|| format!("failed to connect to shell bridge on 127.0.0.1:{port}"))?;
    writeln!(stream, "PIN {pin}")?;
    stream.flush()?;

    println!("Connected to CKA Buddy shell bridge on 127.0.0.1:{port}. Ctrl+C to exit.");

    let mut reader = BufReader::new(stream);
    let shell = default_shell();

    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            println!("Bridge closed.");
            break;
        }

        let command = line.trim();
        if command.is_empty() {
            continue;
        }

        println!("\n$ {command}");
        let status = Command::new(&shell).args(["-lc", command]).status()?;
        println!("[exit: {}]", status.code().unwrap_or(-1));
    }

    Ok(())
}

fn default_shell() -> String {
    env::var("SHELL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "/bin/zsh".to_string())
}

fn generate_pin() -> String {
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    format!("{:04}", ((pid + nanos) % 10_000))
}

fn resolve_terminal_executable(terminal_override: Option<String>) -> Result<String> {
    if let Some(exe) = terminal_override.as_deref() {
        return Ok(exe.to_string());
    }

    if let Ok(exe) = env::var("CKA_BUDDY_TERMINAL")
        && !exe.trim().is_empty() {
            return Ok(exe);
        }

    let candidates = [
        "ghostty",
        "wezterm",
        "alacritty",
        "kitty",
        "gnome-terminal",
        "xterm",
    ];

    for candidate in candidates {
        if is_on_path(candidate) {
            return Ok(candidate.to_string());
        }
    }

    bail!(
        "no supported terminal found (set --terminal or CKA_BUDDY_TERMINAL); tried: ghostty, wezterm, alacritty, kitty, gnome-terminal, xterm"
    )
}

fn is_on_path(bin: &str) -> bool {
    if bin.contains('/') {
        return Path::new(bin).exists();
    }

    let Some(path_var) = env::var_os("PATH") else {
        return false;
    };

    env::split_paths(&path_var).any(|dir| dir.join(bin).exists())
}
