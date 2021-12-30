#[test]
fn test_multiple() {
    use std::io::Read as _;

    let mut pty = pty_process::blocking::Pty::new().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();

    let mut child = pty_process::blocking::Command::new("echo")
        .arg("foo")
        .spawn(&pty)
        .unwrap();
    let mut buf = [0u8; 1024];
    let bytes = pty.read(&mut buf).unwrap();
    assert_eq!(&buf[..bytes], b"foo\r\n");
    let status = child.wait().unwrap();
    assert_eq!(status.code().unwrap(), 0);

    let mut child = pty_process::blocking::Command::new("echo")
        .arg("bar")
        .spawn(&pty)
        .unwrap();
    let mut buf = [0u8; 1024];
    let bytes = pty.read(&mut buf).unwrap();
    assert_eq!(&buf[..bytes], b"bar\r\n");
    let status = child.wait().unwrap();
    assert_eq!(status.code().unwrap(), 0);
}
