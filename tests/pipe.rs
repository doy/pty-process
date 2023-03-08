#[test]
fn test_pipe_basic() {
    let (read_fd, write_fd) = pipe();

    let mut child_from = std::process::Command::new("seq");
    child_from.args(["1", "10"]);
    child_from.stdout(std::process::Stdio::from(write_fd));

    let mut child_to = std::process::Command::new("tac");
    child_to.stdin(std::process::Stdio::from(read_fd));
    child_to.stdout(std::process::Stdio::piped());

    assert!(child_from.status().unwrap().success());
    drop(child_from);
    let output = child_to.output().unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, b"10\n9\n8\n7\n6\n5\n4\n3\n2\n1\n");
}

#[test]
fn test_pipe_blocking() {
    use std::io::Read as _;

    let (read_fd, write_fd) = pipe();

    let pty_from = pty_process::blocking::Pty::new().unwrap();
    let pts_from = pty_from.pts().unwrap();
    pty_from.resize(pty_process::Size::new(24, 80)).unwrap();
    let mut cmd_from = pty_process::blocking::Command::new("seq");
    cmd_from.args(["1", "10"]);
    cmd_from.stdout(std::process::Stdio::from(write_fd));
    let mut child_from = cmd_from.spawn(&pts_from).unwrap();

    let mut pty_to = pty_process::blocking::Pty::new().unwrap();
    let pts_to = pty_to.pts().unwrap();
    let mut cmd_to = pty_process::blocking::Command::new("tac");
    cmd_to.stdin(std::process::Stdio::from(read_fd));
    let mut child_to = cmd_to.spawn(&pts_to).unwrap();

    assert!(child_from.wait().unwrap().success());
    drop(cmd_from);

    // wait for the `tac` process to finish generating output (we don't really
    // have a good way to detect when that happens)
    std::thread::sleep(std::time::Duration::from_millis(100));

    let mut buf = [0u8; 1024];
    let bytes = pty_to.read(&mut buf).unwrap();
    assert_eq!(
        &buf[..bytes],
        b"10\r\n9\r\n8\r\n7\r\n6\r\n5\r\n4\r\n3\r\n2\r\n1\r\n"
    );

    assert!(child_to.wait().unwrap().success());
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_pipe_async() {
    use tokio::io::AsyncReadExt as _;

    let (read_fd, write_fd) = pipe();

    let pty_from = pty_process::Pty::new().unwrap();
    let pts_from = pty_from.pts().unwrap();
    pty_from.resize(pty_process::Size::new(24, 80)).unwrap();
    let mut cmd_from = pty_process::Command::new("seq");
    cmd_from.args(["1", "10"]);
    cmd_from.stdout(std::process::Stdio::from(write_fd));
    let mut child_from = cmd_from.spawn(&pts_from).unwrap();

    let mut pty_to = pty_process::Pty::new().unwrap();
    let pts_to = pty_to.pts().unwrap();
    let mut cmd_to = pty_process::Command::new("tac");
    cmd_to.stdin(std::process::Stdio::from(read_fd));
    let mut child_to = cmd_to.spawn(&pts_to).unwrap();

    assert!(child_from.wait().await.unwrap().success());
    drop(cmd_from);

    // wait for the `tac` process to finish generating output (we
    // don't really have a good way to detect when that happens)
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let mut buf = [0u8; 1024];
    let bytes = pty_to.read(&mut buf).await.unwrap();
    assert_eq!(
        &buf[..bytes],
        b"10\r\n9\r\n8\r\n7\r\n6\r\n5\r\n4\r\n3\r\n2\r\n1\r\n"
    );

    assert!(child_to.wait().await.unwrap().success());
}

fn pipe() -> (std::os::fd::OwnedFd, std::os::fd::OwnedFd) {
    use std::os::fd::FromRawFd as _;

    let (r, w) = nix::unistd::pipe2(nix::fcntl::OFlag::O_CLOEXEC).unwrap();
    (unsafe { std::os::fd::OwnedFd::from_raw_fd(r) }, unsafe {
        std::os::fd::OwnedFd::from_raw_fd(w)
    })
}
