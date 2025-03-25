use nix::libc::O_NONBLOCK;
use nix::sys::stat;
use nix::unistd::{self, unlink};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;

pub const RUNNER_CTL_FIFO: &str = "/tmp/runner.ctl.fifo";
pub const RUNNER_ACK_FIFO: &str = "/tmp/runner.ack.fifo";

pub struct PerfGuard {
    ctl_fifo: FifoIpc,
    ack_fifo: FifoIpc,
}

impl PerfGuard {
    pub fn new(ctl_fifo: &str, ack_fifo: &str) -> Option<Self> {
        let mut instance = Self {
            ctl_fifo: FifoIpc::connect(ctl_fifo)?.with_writer().ok()?,
            ack_fifo: FifoIpc::connect(ack_fifo)?.with_reader().ok()?,
        };
        instance.send_cmd(Command::StartBenchmark)?;
        Some(instance)
    }

    fn send_cmd(&mut self, cmd: Command) -> Option<()> {
        self.ctl_fifo.send_cmd(cmd)?;
        self.ack_fifo.wait_for_ack()?;

        Some(())
    }
}

impl Drop for PerfGuard {
    fn drop(&mut self) {
        self.send_cmd(Command::StopBenchmark);
    }
}

pub struct FifoIpc {
    path: PathBuf,
    reader: Option<File>,
    writer: Option<File>,
}

impl FifoIpc {
    pub fn connect<P: Into<PathBuf>>(path: P) -> Option<Self> {
        let path = path.into();

        if !path.exists() {
            return None;
        }

        Some(Self {
            path,
            reader: None,
            writer: None,
        })
    }

    pub fn create(path: &str) -> Option<Self> {
        // Remove the previous FIFO (if it exists)
        let _ = unlink(path);

        // Create the FIFO with RWX permissions for the owner
        unistd::mkfifo(path, stat::Mode::S_IRWXU).unwrap();

        Self::connect(path)
    }

    pub fn with_reader(mut self) -> std::io::Result<Self> {
        self.reader = Some(
            OpenOptions::new()
                .write(true)
                .read(true)
                .custom_flags(O_NONBLOCK)
                .open(&self.path)?,
        );
        Ok(self)
    }

    /// WARNING: Writer must be opened _AFTER_ the reader.
    pub fn with_writer(mut self) -> std::io::Result<Self> {
        self.writer = Some(
            OpenOptions::new()
                .write(true)
                .custom_flags(O_NONBLOCK)
                .open(&self.path)?,
        );
        Ok(self)
    }

    pub fn recv_cmd(&mut self) -> Option<Command> {
        // First read the length (u32 = 4 bytes)
        let mut len_buffer = [0u8; 4];
        self.read_exact(&mut len_buffer).ok()?;
        let message_len = u32::from_le_bytes(len_buffer) as usize;

        // Try to read the message
        let mut buffer = vec![0u8; message_len];
        loop {
            if self.read_exact(&mut buffer).is_ok() {
                break;
            }
        }

        let decoded = bincode::deserialize(&buffer).ok()?;
        Some(decoded)
    }

    pub fn send_cmd(&mut self, cmd: Command) -> Option<()> {
        let encoded = bincode::serialize(&cmd).ok()?;
        self.write_all(&(encoded.len() as u32).to_le_bytes()).ok()?;
        self.write_all(&encoded).ok()?;
        Some(())
    }

    pub fn wait_for_ack(&mut self) -> Option<()> {
        // Wait for ACK command
        loop {
            if let Some(Command::Ack) = self.recv_cmd() {
                break;
            }
        }

        Some(())
    }
}

impl std::io::Write for FifoIpc {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Some(writer) = self.writer.as_mut() {
            writer.write(buf)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Writer not initialized",
            ))
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl std::io::Read for FifoIpc {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if let Some(reader) = self.reader.as_mut() {
            reader.read(buf)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Reader not initialized",
            ))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Command {
    StartBenchmark,
    StopBenchmark,
    Ack,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_write_read() {
        let mut fifo = FifoIpc::create("/tmp/test1.fifo")
            .unwrap()
            .with_reader()
            .unwrap()
            .with_writer()
            .unwrap();

        fifo.write_all(b"Hello").unwrap();
        let mut buffer = [0; 5];
        fifo.read_exact(&mut buffer).unwrap();
        assert_eq!(&buffer, b"Hello");
    }

    #[test]
    fn test_ipc_send_recv_cmd() {
        let mut fifo = FifoIpc::create("/tmp/test2.fifo")
            .unwrap()
            .with_reader()
            .unwrap()
            .with_writer()
            .unwrap();

        fifo.send_cmd(Command::StartBenchmark).unwrap();
        let cmd = fifo.recv_cmd().unwrap();
        assert_eq!(cmd, Command::StartBenchmark);
    }
}
