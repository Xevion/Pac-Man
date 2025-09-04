#![allow(dead_code)]

//! Buffered tracing setup for handling logs before console attachment.

use crate::platform::buffered_writer::BufferedWriter;
use std::io;
use tracing::{debug, Level};
use tracing_error::ErrorLayer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;

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

            // Flush any buffered content
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

/// Sets up a switchable tracing subscriber that can transition from buffered to direct output.
///
/// Returns the switchable writer that can be used to control the behavior.
pub fn setup_switchable_subscriber() -> SwitchableWriter {
    let switchable_writer = SwitchableWriter::default();
    let make_writer = SwitchableMakeWriter::new(switchable_writer.clone());

    let _subscriber = tracing_subscriber::fmt()
        .with_ansi(cfg!(not(target_os = "emscripten")))
        .with_max_level(Level::DEBUG)
        .with_writer(make_writer)
        .finish()
        .with(ErrorLayer::default());

    tracing::subscriber::set_global_default(_subscriber).expect("Could not set global default switchable subscriber");

    switchable_writer
}
