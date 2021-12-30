#[test]
fn test_fds() {
    use std::io::BufRead as _;

    check_open_fds();

    let pty = pty_process::blocking::Pty::new().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();
    let mut child = pty_process::blocking::Command::new("perl")
        .arg("-Efor my $fd (0..255) { open my $fh, \"<&=$fd\"; print $fd if stat $fh }; say")
        .spawn(&pty)
        .unwrap();
    let mut buf = vec![];
    std::io::BufReader::new(&pty)
        .read_until(b'\n', &mut buf)
        .unwrap();
    assert_eq!(&buf, b"012\r\n");
    let status = child.wait().unwrap();
    assert_eq!(status.code().unwrap(), 0);
    drop(pty);
    check_open_fds();

    let pty = pty_process::blocking::Pty::new().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();
    let mut child = pty_process::blocking::Command::new("perl")
        .arg("-Efor my $fd (0..255) { open my $fh, \"<&=$fd\"; print $fd if stat $fh }; say")
        .stderr(Some(std::process::Stdio::null()))
        .spawn(&pty)
        .unwrap();
    let mut buf = vec![];
    std::io::BufReader::new(&pty)
        .read_until(b'\n', &mut buf)
        .unwrap();
    assert_eq!(&buf, b"012\r\n");
    let status = child.wait().unwrap();
    assert_eq!(status.code().unwrap(), 0);
    drop(pty);
    check_open_fds();
}

#[track_caller]
fn check_open_fds() {
    let open: Vec<_> = (0..=255)
        .filter(|fd| nix::sys::stat::fstat(*fd).is_ok())
        .collect();
    assert_eq!(open, vec![0, 1, 2]);
}
