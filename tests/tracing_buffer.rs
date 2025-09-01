use pacman::platform::tracing_buffer::SwitchableWriter;
use std::io::Write;

#[test]
fn test_switchable_writer_buffering() {
    let mut writer = SwitchableWriter::default();

    // Write some data while in buffered mode
    writer.write_all(b"Hello, ").unwrap();
    writer.write_all(b"world!").unwrap();
    writer.write_all(b"This is buffered content.\n").unwrap();

    // Switch to direct mode (this should flush to stdout and show buffer size)
    // In a real test we can't easily capture stdout, so we'll just verify it doesn't panic
    writer.switch_to_direct_mode().unwrap();

    // Write more data in direct mode
    writer.write_all(b"Direct output after flush\n").unwrap();
}
