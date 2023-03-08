mod helpers;

#[test]
fn test_multiple() {
    let pty = pty_process::blocking::Pty::new().unwrap();
    let pts = pty.pts().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();

    let mut child = pty_process::blocking::Command::new("echo")
        .arg("foo")
        .spawn(&pts)
        .unwrap();

    let mut output = helpers::output(&pty);
    assert_eq!(output.next().unwrap(), "foo\r\n");

    let status = child.wait().unwrap();
    assert_eq!(status.code().unwrap(), 0);

    let mut child = pty_process::blocking::Command::new("echo")
        .arg("bar")
        .spawn(&pts)
        .unwrap();

    assert_eq!(output.next().unwrap(), "bar\r\n");

    let status = child.wait().unwrap();
    assert_eq!(status.code().unwrap(), 0);
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_multiple_async() {
    use futures::stream::StreamExt as _;

    let mut pty = pty_process::Pty::new().unwrap();
    let pts = pty.pts().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();

    let mut child = pty_process::Command::new("echo")
        .arg("foo")
        .spawn(&pts)
        .unwrap();
    let (pty_r, _) = pty.split();

    let mut output = helpers::output_async(pty_r);
    assert_eq!(output.next().await.unwrap(), "foo\r\n");

    let status = child.wait().await.unwrap();
    assert_eq!(status.code().unwrap(), 0);

    let mut child = pty_process::Command::new("echo")
        .arg("bar")
        .spawn(&pts)
        .unwrap();

    assert_eq!(output.next().await.unwrap(), "bar\r\n");

    let status = child.wait().await.unwrap();
    assert_eq!(status.code().unwrap(), 0);
}

#[test]
fn test_multiple_configured() {
    use std::io::BufRead as _;
    use std::os::fd::AsRawFd as _;

    let pty = pty_process::blocking::Pty::new().unwrap();
    let pts = pty.pts().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();

    let (stderr_pipe_r, stderr_pipe_w) = pipe();
    let mut stderr_pipe_r =
        std::io::BufReader::new(std::fs::File::from(stderr_pipe_r));
    let (pre_exec_pipe_r, pre_exec_pipe_w) = pipe();
    let mut pre_exec_pipe_r =
        std::io::BufReader::new(std::fs::File::from(pre_exec_pipe_r));
    let mut cmd = pty_process::blocking::Command::new("perl");
    cmd.arg("-Esay 'foo'; say STDERR 'foo-stderr'; open my $fh, '>&=3'; say $fh 'foo-3';")
        .stderr(std::process::Stdio::from(stderr_pipe_w));
    unsafe {
        cmd.pre_exec(move || {
            nix::unistd::dup2(pre_exec_pipe_w.as_raw_fd(), 3)?;
            nix::fcntl::fcntl(
                3,
                nix::fcntl::F_SETFD(nix::fcntl::FdFlag::empty()),
            )?;
            Ok(())
        });
    }
    let mut child = cmd.spawn(&pts).unwrap();

    let mut output = helpers::output(&pty);
    assert_eq!(output.next().unwrap(), "foo\r\n");

    let mut buf = vec![];
    nix::unistd::alarm::set(5);
    stderr_pipe_r.read_until(b'\n', &mut buf).unwrap();
    nix::unistd::alarm::cancel();
    assert_eq!(std::string::String::from_utf8(buf).unwrap(), "foo-stderr\n");

    let mut buf = vec![];
    nix::unistd::alarm::set(5);
    pre_exec_pipe_r.read_until(b'\n', &mut buf).unwrap();
    nix::unistd::alarm::cancel();
    assert_eq!(std::string::String::from_utf8(buf).unwrap(), "foo-3\n");

    let status = child.wait().unwrap();
    assert_eq!(status.code().unwrap(), 0);

    let mut child = cmd.spawn(&pts).unwrap();
    let mut output = helpers::output(&pty);

    assert_eq!(output.next().unwrap(), "foo\r\n");

    let mut buf = vec![];
    nix::unistd::alarm::set(5);
    stderr_pipe_r.read_until(b'\n', &mut buf).unwrap();
    nix::unistd::alarm::cancel();
    assert_eq!(std::string::String::from_utf8(buf).unwrap(), "foo-stderr\n");

    let mut buf = vec![];
    nix::unistd::alarm::set(5);
    pre_exec_pipe_r.read_until(b'\n', &mut buf).unwrap();
    nix::unistd::alarm::cancel();
    assert_eq!(std::string::String::from_utf8(buf).unwrap(), "foo-3\n");

    let status = child.wait().unwrap();
    assert_eq!(status.code().unwrap(), 0);
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_multiple_configured_async() {
    use futures::stream::StreamExt as _;
    use std::os::fd::{AsRawFd as _, FromRawFd as _, IntoRawFd as _};
    use tokio::io::AsyncBufReadExt as _;

    let mut pty = pty_process::Pty::new().unwrap();
    let pts = pty.pts().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();
    let (pty_r, _) = pty.split();

    let (stderr_pipe_r, stderr_pipe_w) = pipe();
    let mut stderr_pipe_r = tokio::io::BufReader::new(unsafe {
        tokio::fs::File::from_raw_fd(stderr_pipe_r.into_raw_fd())
    });
    let (pre_exec_pipe_r, pre_exec_pipe_w) = pipe();
    let mut pre_exec_pipe_r = tokio::io::BufReader::new(unsafe {
        tokio::fs::File::from_raw_fd(pre_exec_pipe_r.into_raw_fd())
    });
    let mut cmd = pty_process::Command::new("perl");
    cmd.arg(
        "-Esay 'foo'; \
            say STDERR 'foo-stderr'; \
            open my $fh, '>&=3'; \
            say $fh 'foo-3';",
    )
    .stderr(std::process::Stdio::from(stderr_pipe_w));
    unsafe {
        cmd.pre_exec(move || {
            nix::unistd::dup2(pre_exec_pipe_w.as_raw_fd(), 3)?;
            nix::fcntl::fcntl(
                3,
                nix::fcntl::F_SETFD(nix::fcntl::FdFlag::empty()),
            )?;
            Ok(())
        });
    }
    let mut child = cmd.spawn(&pts).unwrap();

    let mut output = helpers::output_async(pty_r);
    assert_eq!(output.next().await.unwrap(), "foo\r\n");

    let mut buf = vec![];
    tokio::time::timeout(
        std::time::Duration::from_secs(5),
        stderr_pipe_r.read_until(b'\n', &mut buf),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(std::string::String::from_utf8(buf).unwrap(), "foo-stderr\n");

    let mut buf = vec![];
    tokio::time::timeout(
        std::time::Duration::from_secs(5),
        pre_exec_pipe_r.read_until(b'\n', &mut buf),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(std::string::String::from_utf8(buf).unwrap(), "foo-3\n");

    let status = child.wait().await.unwrap();
    assert_eq!(status.code().unwrap(), 0);

    let mut child = cmd.spawn(&pts).unwrap();

    assert_eq!(output.next().await.unwrap(), "foo\r\n");

    let mut buf = vec![];
    tokio::time::timeout(
        std::time::Duration::from_secs(5),
        stderr_pipe_r.read_until(b'\n', &mut buf),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(std::string::String::from_utf8(buf).unwrap(), "foo-stderr\n");

    let mut buf = vec![];
    tokio::time::timeout(
        std::time::Duration::from_secs(5),
        pre_exec_pipe_r.read_until(b'\n', &mut buf),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(std::string::String::from_utf8(buf).unwrap(), "foo-3\n");

    let status = child.wait().await.unwrap();
    assert_eq!(status.code().unwrap(), 0);
}

#[test]
fn test_controlling_terminal() {
    let pty = pty_process::blocking::Pty::new().unwrap();
    let pts = pty.pts().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();
    let mut child = pty_process::blocking::Command::new("perl")
        .arg("-Eopen my $fh, '<', '/dev/tty' or die; if (-t $fh) { say 'true' } else { say 'false' }")
        .spawn(&pts)
        .unwrap();

    let mut output = helpers::output(&pty);
    assert_eq!(output.next().unwrap(), "true\r\n");

    let status = child.wait().unwrap();
    assert_eq!(status.code().unwrap(), 0);
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_controlling_terminal_async() {
    use futures::stream::StreamExt as _;

    let mut pty = pty_process::Pty::new().unwrap();
    let pts = pty.pts().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();
    let (pty_r, _) = pty.split();
    let mut child = pty_process::Command::new("perl")
        .arg(
            "-Eopen my $fh, '<', '/dev/tty' or die; \
                if (-t $fh) { say 'true' } else { say 'false' }",
        )
        .spawn(&pts)
        .unwrap();

    let mut output = helpers::output_async(pty_r);
    assert_eq!(output.next().await.unwrap(), "true\r\n");

    let status = child.wait().await.unwrap();
    assert_eq!(status.code().unwrap(), 0);
}

#[test]
fn test_session_leader() {
    let pty = pty_process::blocking::Pty::new().unwrap();
    let pts = pty.pts().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();
    let mut child = pty_process::blocking::Command::new("python")
        .arg("-cimport os; print(os.getpid() == os.getsid(0))")
        .spawn(&pts)
        .unwrap();

    let mut output = helpers::output(&pty);
    assert_eq!(output.next().unwrap(), "True\r\n");

    let status = child.wait().unwrap();
    assert_eq!(status.code().unwrap(), 0);
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_session_leader_async() {
    use futures::stream::StreamExt as _;

    let mut pty = pty_process::Pty::new().unwrap();
    let pts = pty.pts().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();
    let mut child = pty_process::Command::new("python")
        .arg("-cimport os; print(os.getpid() == os.getsid(0))")
        .spawn(&pts)
        .unwrap();

    let (pty_r, _) = pty.split();
    let mut output = helpers::output_async(pty_r);
    assert_eq!(output.next().await.unwrap(), "True\r\n");

    let status = child.wait().await.unwrap();
    eprintln!("{:?}", status);
    assert_eq!(status.code().unwrap(), 0);
}

fn pipe() -> (std::os::fd::OwnedFd, std::os::fd::OwnedFd) {
    use std::os::fd::FromRawFd as _;

    let (r, w) = nix::unistd::pipe().unwrap();
    (unsafe { std::os::fd::OwnedFd::from_raw_fd(r) }, unsafe {
        std::os::fd::OwnedFd::from_raw_fd(w)
    })
}
