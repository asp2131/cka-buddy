use std::io::{Read, Write};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use anyhow::Result;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};

pub struct PtySession {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    output_rx: mpsc::Receiver<String>,
    _child: Box<dyn portable_pty::Child + Send>,
}

impl PtySession {
    pub fn start(shell_path: &str) -> Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows: 40,
            cols: 140,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let cmd = CommandBuilder::new(shell_path);
        let child = pair.slave.spawn_command(cmd)?;
        drop(pair.slave);

        let mut reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        let writer = Arc::new(Mutex::new(writer));
        let (tx, rx) = mpsc::channel::<String>();

        thread::spawn(move || {
            let mut buf = [0_u8; 8192];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let text = String::from_utf8_lossy(&buf[..n]).to_string();
                        let _ = tx.send(text);
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            writer,
            output_rx: rx,
            _child: child,
        })
    }

    pub fn send_line(&mut self, line: &str) -> Result<()> {
        let mut writer = self.writer.lock().expect("pty writer poisoned");
        writer.write_all(line.as_bytes())?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        Ok(())
    }

    pub fn drain_output(&mut self) -> Vec<String> {
        let mut out = Vec::new();
        while let Ok(chunk) = self.output_rx.try_recv() {
            out.extend(chunk.lines().map(ToString::to_string));
        }
        out
    }
}
