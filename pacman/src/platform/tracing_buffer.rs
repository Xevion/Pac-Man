//! Buffered tracing setup for handling logs before console attachment.

use parking_lot::Mutex;
use std::io;
use std::io::Write;
use std::sync::Arc;
use tracing::debug;
use tracing_subscriber::fmt::MakeWriter;

/// A thread-safe buffered writer that stores logs in memory until flushed.
#[derive(Clone)]
pub struct BufferedWriter {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl BufferedWriter {
    /// Creates a new buffered writer.
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Flushes all buffered content to the provided writer and clears the buffer.
    pub fn flush_to<W: Write>(&self, mut writer: W) -> io::Result<()> {
        let mut buffer = self.buffer.lock();
        if !buffer.is_empty() {
            writer.write_all(&buffer)?;
            writer.flush()?;
            buffer.clear();
        }
        Ok(())
    }

    /// Returns the current buffer size in bytes.
    pub fn buffer_size(&self) -> usize {
        self.buffer.lock().len()
    }
}

impl Write for BufferedWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut buffer = self.buffer.lock();
        buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        // For buffered writer, flush is a no-op since we're storing in memory
        Ok(())
    }
}

impl Default for BufferedWriter {
    fn default() -> Self {
        Self::new()
    }
}

/// A writer that can switch between buffering and direct output.
#[derive(Clone, Default)]
pub struct SwitchableWriter {
    buffered_writer: BufferedWriter,
    direct_mode: std::sync::Arc<parking_lot::Mutex<bool>>,
}

impl SwitchableWriter {
    pub fn switch_to_direct_mode(&self) -> io::Result<()> {
        let buffer_size = {
            // Acquire the lock
            let mut mode = self.direct_mode.lock();

            // Get buffer size before flushing for debug logging
            let buffer_size = self.buffered_writer.buffer_size();

            // Flush any buffered content to stdout only
            self.buffered_writer.flush_to(io::stdout())?;

            // Switch to direct mode (and drop the lock)
            *mode = true;

            buffer_size
        };

        // Log how much was buffered (this will now go directly to stdout)
        debug!("Flushed {buffer_size:?} bytes of buffered logs to console");

        Ok(())
    }
}

impl io::Write for SwitchableWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if *self.direct_mode.lock() {
            io::stdout().write(buf)
        } else {
            self.buffered_writer.clone().write(buf)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        if *self.direct_mode.lock() {
            io::stdout().flush()
        } else {
            // For buffered mode, flush is a no-op
            Ok(())
        }
    }
}

/// A make writer that uses the switchable writer.
#[derive(Clone)]
pub struct SwitchableMakeWriter {
    writer: SwitchableWriter,
}

impl SwitchableMakeWriter {
    pub fn new(writer: SwitchableWriter) -> Self {
        Self { writer }
    }
}

impl<'a> MakeWriter<'a> for SwitchableMakeWriter {
    type Writer = SwitchableWriter;

    fn make_writer(&'a self) -> Self::Writer {
        self.writer.clone()
    }
}
