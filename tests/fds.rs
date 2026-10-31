mod helpers;

#[test]
fn test_fds() {
    // On illumos, grantpt(3C) looks up the tty group, which leaves libc
    // holding a close-on-exec door to nscd for the rest of the process. Open
    // one pty before taking the baseline so that door is already in it.
    drop(pty_process::blocking::open().unwrap());
    let fds = check_open_fds(None);
    // The child inherits only the descriptors that survive exec.
    let fds_strings: Vec<String> = fds
        .iter()
        .filter(|fd| !is_cloexec(**fd))
        .map(std::string::ToString::to_string)
        .collect();
    let expected_output = format!("{}\r\n", fds_strings.join(""));

    let (pty, pts) = pty_process::blocking::open().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();
    let mut child = pty_process::blocking::Command::new("perl")
        .arg("-Efor my $fd (0..255) { open my $fh, \"<&=$fd\"; print $fd if stat $fh }; say")
        .spawn(pts)
        .unwrap();

    let mut output = helpers::output(&pty);
    assert_eq!(output.next().unwrap(), expected_output);

    let status = child.wait().unwrap();
    assert_eq!(status.code().unwrap(), 0);
    drop(pty);
    check_open_fds(Some(&fds));

    let (pty, pts) = pty_process::blocking::open().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();
    let mut child = pty_process::blocking::Command::new("perl")
        .arg("-Efor my $fd (0..255) { open my $fh, \"<&=$fd\"; print $fd if stat $fh }; say")
        .stderr(std::process::Stdio::null())
        .spawn(pts)
        .unwrap();

    let mut output = helpers::output(&pty);
    assert_eq!(output.next().unwrap(), expected_output);

    let status = child.wait().unwrap();
    assert_eq!(status.code().unwrap(), 0);
    drop(pty);
    check_open_fds(Some(&fds));
}

fn is_cloexec(fd: i32) -> bool {
    let flags = nix::fcntl::fcntl(
        unsafe { std::os::fd::BorrowedFd::borrow_raw(fd) },
        nix::fcntl::FcntlArg::F_GETFD,
    )
    .unwrap();
    nix::fcntl::FdFlag::from_bits_truncate(flags)
        .contains(nix::fcntl::FdFlag::FD_CLOEXEC)
}

#[track_caller]
fn check_open_fds(expected: Option<&[i32]>) -> Vec<i32> {
    let open: Vec<_> = (0..=255)
        .filter(|fd| {
            nix::sys::stat::fstat(unsafe {
                std::os::fd::BorrowedFd::borrow_raw(*fd)
            })
            .is_ok()
        })
        .collect();
    assert!(open.contains(&0));
    assert!(open.contains(&1));
    assert!(open.contains(&2));
    if let Some(fds) = expected {
        assert_eq!(open, fds);
    }
    open.to_vec()
}
