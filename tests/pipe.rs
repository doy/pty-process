#[test]
fn test_pipe_basic() {
    use std::os::unix::io::FromRawFd as _;

    let (read_fd, write_fd) = nix::unistd::pipe().unwrap();

    let mut child_from = std::process::Command::new("seq");
    child_from.args(["1", "10"]);
    child_from.stdout(unsafe { std::process::Stdio::from_raw_fd(write_fd) });

    let mut child_to = std::process::Command::new("tac");
    child_to.stdin(unsafe { std::process::Stdio::from_raw_fd(read_fd) });
    child_to.stdout(std::process::Stdio::piped());

    assert!(child_from.status().unwrap().success());
    nix::unistd::close(write_fd).unwrap();
    let output = child_to.output().unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, b"10\n9\n8\n7\n6\n5\n4\n3\n2\n1\n");
}

#[cfg(feature = "todo")]
// TODO (hangs because i'm still overriding the configured fds)
// #[test]
fn test_pipe_async() {
    use async_std::io::ReadExt as _;
    use std::os::unix::io::FromRawFd as _;

    let (status_from, status_to) = async_std::task::block_on(async {
        let (read_fd, write_fd) = nix::unistd::pipe().unwrap();

        let pty_from = pty_process::async_std::Pty::new().unwrap();
        pty_from.resize(pty_process::Size::new(24, 80)).unwrap();
        let mut cmd_from = pty_process::async_std::Command::new("seq");
        cmd_from.args(["1", "10"]);
        cmd_from
            .stdout(unsafe { std::process::Stdio::from_raw_fd(write_fd) });
        let mut child_from = cmd_from.spawn(pty_from).unwrap();

        let pty_to = pty_process::async_std::Pty::new().unwrap();
        pty_to.resize(pty_process::Size::new(24, 80)).unwrap();
        let mut cmd_to = pty_process::async_std::Command::new("tac");
        cmd_to.stdin(unsafe { std::process::Stdio::from_raw_fd(read_fd) });
        let mut child_to = cmd_to.spawn(pty_to).unwrap();

        // the pty will echo the written bytes back immediately, but the
        // subprocess needs to generate its own output, which takes time, so
        // we can't just read immediately (we may just get the echoed bytes).
        // because the output generation is happening in the subprocess, we
        // also don't have any way to know when (or if!) the subprocess will
        // decide to send its output, so sleeping is the best we can do.
        async_std::task::sleep(std::time::Duration::from_secs(1)).await;

        let mut buf = [0u8; 1024];
        let bytes = child_to.pty().read(&mut buf).await.unwrap();
        assert_eq!(
            &buf[..bytes],
            b"10\r\n9\r\n8\r\n7\r\n6\r\n5\r\n4\r\n3\r\n2\r\n1\r\n"
        );

        (
            child_from.status().await.unwrap(),
            child_to.status().await.unwrap(),
        )
    });
    assert_eq!(status_from.code().unwrap(), 0);
    assert_eq!(status_to.code().unwrap(), 0);
}
