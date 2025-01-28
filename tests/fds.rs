mod helpers;

#[test]
fn test_fds() {
    let fds = check_open_fds(None);
    let fds_strings: Vec<String> =
        fds.iter().map(std::string::ToString::to_string).collect();
    let expected_output = format!("{}\r\n", fds_strings.join(""));

    let (pty, pts) = pty_process::blocking::open().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();
    let mut child = pty_process::blocking::Command::new("perl")
        .arg("-Efor my $fd (0..255) { open my $fh, \"<&=$fd\"; print $fd if stat $fh }; say")
        .spawn(&pts)
        .unwrap();

    let mut output = helpers::output(&pty);
    assert_eq!(output.next().unwrap(), expected_output);

    let status = child.wait().unwrap();
    assert_eq!(status.code().unwrap(), 0);
    drop(pty);
    drop(pts);
    check_open_fds(Some(&fds));

    let (pty, pts) = pty_process::blocking::open().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();
    let mut child = pty_process::blocking::Command::new("perl")
        .arg("-Efor my $fd (0..255) { open my $fh, \"<&=$fd\"; print $fd if stat $fh }; say")
        .stderr(std::process::Stdio::null())
        .spawn(&pts)
        .unwrap();

    let mut output = helpers::output(&pty);
    assert_eq!(output.next().unwrap(), expected_output);

    let status = child.wait().unwrap();
    assert_eq!(status.code().unwrap(), 0);
    drop(pty);
    drop(pts);
    check_open_fds(Some(&fds));
}

#[track_caller]
fn check_open_fds(expected: Option<&[i32]>) -> Vec<i32> {
    let open: Vec<_> = (0..=255)
        .filter(|fd| nix::sys::stat::fstat(*fd).is_ok())
        .collect();
    assert!(open.contains(&0));
    assert!(open.contains(&1));
    assert!(open.contains(&2));
    if let Some(fds) = expected {
        assert_eq!(open, fds);
    }
    open.to_vec()
}
