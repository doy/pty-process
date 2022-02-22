mod helpers;

#[test]
fn test_fds() {
    check_open_fds();

    let pty = pty_process::blocking::Pty::new().unwrap();
    let pts = pty.pts().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();
    let mut child = pty_process::blocking::Command::new("perl")
        .arg("-Efor my $fd (0..255) { open my $fh, \"<&=$fd\"; print $fd if stat $fh }; say")
        .spawn(&pts)
        .unwrap();

    let mut output = helpers::output(&pty);
    assert_eq!(output.next().unwrap(), "012\r\n");

    let status = child.wait().unwrap();
    assert_eq!(status.code().unwrap(), 0);
    drop(pty);
    drop(pts);
    check_open_fds();

    let pty = pty_process::blocking::Pty::new().unwrap();
    let pts = pty.pts().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();
    let mut child = pty_process::blocking::Command::new("perl")
        .arg("-Efor my $fd (0..255) { open my $fh, \"<&=$fd\"; print $fd if stat $fh }; say")
        .stderr(std::process::Stdio::null())
        .spawn(&pts)
        .unwrap();

    let mut output = helpers::output(&pty);
    assert_eq!(output.next().unwrap(), "012\r\n");

    let status = child.wait().unwrap();
    assert_eq!(status.code().unwrap(), 0);
    drop(pty);
    drop(pts);
    check_open_fds();
}

#[track_caller]
fn check_open_fds() {
    let open: Vec<_> = (0..=255)
        .filter(|fd| nix::sys::stat::fstat(*fd).is_ok())
        .collect();
    assert_eq!(open, vec![0, 1, 2]);
}
