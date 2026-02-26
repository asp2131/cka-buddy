use std::env;
use std::path::Path;
use std::process::Command;
use std::thread;

use anyhow::{Result, bail};

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
    shell_path: String,
    terminal_override: Option<String>,
    external_output: Vec<String>,
}

impl ShellRouter {
    pub fn new(mode: ShellMode, embedded: PtySession, terminal_override: Option<String>) -> Self {
        Self {
            mode,
            embedded,
            shell_path: "/bin/zsh".to_string(),
            terminal_override,
            external_output: Vec::new(),
        }
    }

    pub fn mode(&self) -> ShellMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: ShellMode) {
        self.mode = mode;
    }

    pub fn send_line(&mut self, line: &str) -> Result<String> {
        match self.mode {
            ShellMode::Embedded => {
                self.embedded.send_line(line)?;
                Ok(format!("embedded dispatch: {line}"))
            }
            ShellMode::External => {
                let terminal = self.resolve_terminal_executable()?;
                let cmd_payload = format!("{}; exec {}", line, self.shell_path);

                let mut command = Command::new(&terminal);
                match terminal.as_str() {
                    "wezterm" => {
                        command.args(["start", "--", &self.shell_path, "-lc", &cmd_payload]);
                    }
                    "gnome-terminal" => {
                        command.args(["--", &self.shell_path, "-lc", &cmd_payload]);
                    }
                    _ => {
                        command.args(["-e", &self.shell_path, "-lc", &cmd_payload]);
                    }
                }

                let mut child = command.spawn()?;
                let pid = child.id();
                thread::spawn(move || {
                    let _ = child.wait();
                });
                let msg = format!("external dispatch via {terminal} (pid {pid}): {line}");
                self.external_output.push(msg.clone());
                Ok(msg)
            }
        }
    }

    pub fn drain_output(&mut self) -> Vec<String> {
        match self.mode {
            ShellMode::Embedded => self.embedded.drain_output(),
            ShellMode::External => {
                let mut out = Vec::new();
                std::mem::swap(&mut out, &mut self.external_output);
                out
            }
        }
    }

    fn resolve_terminal_executable(&self) -> Result<String> {
        if let Some(exe) = self.terminal_override.as_deref() {
            return Ok(exe.to_string());
        }

        if let Ok(exe) = env::var("CKA_BUDDY_TERMINAL") {
            if !exe.trim().is_empty() {
                return Ok(exe);
            }
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
