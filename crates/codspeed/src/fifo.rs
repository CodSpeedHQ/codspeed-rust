pub use super::shared::*;
use anyhow::bail;
use nix::libc::O_NONBLOCK;
use nix::sys::stat;
use nix::unistd::{self, unlink};
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

pub struct BenchGuard {
    ctl_fifo: FifoIpc,
    ack_fifo: FifoIpc,
}

impl BenchGuard {
    pub fn new(ctl_fifo: &str, ack_fifo: &str) -> anyhow::Result<Self> {
        let mut instance = Self {
            ctl_fifo: FifoIpc::connect(ctl_fifo)?.with_writer()?,
            ack_fifo: FifoIpc::connect(ack_fifo)?.with_reader()?,
        };

        instance.send_cmd(Command::SetIntegration {
            name: "codspeed-rust".into(),
            version: env!("CARGO_PKG_VERSION").into(),
        })?; // FIXME: Just send it once
        instance.send_cmd(Command::StartBenchmark)?;

        Ok(instance)
    }

    pub fn new_with_runner_fifo() -> anyhow::Result<Self> {
        Self::new(RUNNER_CTL_FIFO, RUNNER_ACK_FIFO)
    }

    fn send_cmd(&mut self, cmd: Command) -> anyhow::Result<()> {
        self.ctl_fifo.send_cmd(cmd)?;
        self.ack_fifo.wait_for_ack();

        Ok(())
    }
}

impl Drop for BenchGuard {
    fn drop(&mut self) {
        self.send_cmd(Command::StopBenchmark)
            .expect("Failed to send stop command");
    }
}

pub fn send_cmd(cmd: Command) -> anyhow::Result<()> {
    let mut writer = FifoIpc::connect(RUNNER_CTL_FIFO)?.with_writer()?;
    writer.send_cmd(cmd).unwrap();

    let mut reader = FifoIpc::connect(RUNNER_ACK_FIFO)?.with_reader()?;
    reader.wait_for_ack();

    Ok(())
}

pub struct FifoIpc {
    path: PathBuf,
    reader: Option<File>,
    writer: Option<File>,
}

impl FifoIpc {
    /// Creates a new FIFO at the specified path and connects to it.
    ///
    /// ```rust
    /// use codspeed::fifo::{FifoIpc, Command};
    ///
    /// // Create the reader before the writer (required!):
    /// let mut read_fifo = FifoIpc::create("/tmp/doctest.fifo").unwrap().with_reader().unwrap();
    ///
    /// // Connect to the FIFO and send a command
    /// let mut fifo = FifoIpc::connect("/tmp/doctest.fifo").unwrap().with_writer().unwrap();
    /// fifo.send_cmd(Command::StartBenchmark).unwrap();
    ///
    /// // Receive the command in the reader
    /// let cmd = read_fifo.recv_cmd().unwrap();
    /// assert_eq!(cmd, Command::StartBenchmark);
    /// ```
    pub fn create<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        // Remove the previous FIFO (if it exists)
        let _ = unlink(path.as_ref());

        // Create the FIFO with RWX permissions for the owner
        unistd::mkfifo(path.as_ref(), stat::Mode::S_IRWXU)?;

        Self::connect(path.as_ref())
    }

    pub fn connect<P: Into<PathBuf>>(path: P) -> anyhow::Result<Self> {
        let path = path.into();

        if !path.exists() {
            bail!("FIFO does not exist: {}", path.display());
        }

        Ok(Self {
            path,
            reader: None,
            writer: None,
        })
    }

    pub fn with_reader(mut self) -> anyhow::Result<Self> {
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
    pub fn with_writer(mut self) -> anyhow::Result<Self> {
        self.writer = Some(
            OpenOptions::new()
                .write(true)
                .custom_flags(O_NONBLOCK)
                .open(&self.path)?,
        );
        Ok(self)
    }

    pub fn recv_cmd(&mut self) -> anyhow::Result<Command> {
        // First read the length (u32 = 4 bytes)
        let mut len_buffer = [0u8; 4];
        self.read_exact(&mut len_buffer)?;
        let message_len = u32::from_le_bytes(len_buffer) as usize;

        // Try to read the message
        let mut buffer = vec![0u8; message_len];
        loop {
            if self.read_exact(&mut buffer).is_ok() {
                break;
            }
        }

        let decoded = bincode::deserialize(&buffer)?;
        Ok(decoded)
    }

    pub fn send_cmd(&mut self, cmd: Command) -> anyhow::Result<()> {
        use std::io::Write;

        let encoded = bincode::serialize(&cmd)?;
        self.write_all(&(encoded.len() as u32).to_le_bytes())?;
        self.write_all(&encoded)?;
        Ok(())
    }

    pub fn wait_for_ack(&mut self) {
        loop {
            if let Ok(Command::Ack) = self.recv_cmd() {
                break;
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

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
