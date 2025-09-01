//! Buffered writer for tracing logs that can store logs before console attachment.

use parking_lot::Mutex;
use std::io::{self, Write};
use std::sync::Arc;

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
