mod helpers;

#[test]
fn test_multiple() {
    let pty = pty_process::blocking::Pty::new().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();

    let mut child = pty_process::blocking::Command::new("echo")
        .arg("foo")
        .spawn(&pty)
        .unwrap();

    let mut output = helpers::output(&pty);
    assert_eq!(output.next().unwrap(), "foo\r\n");

    let status = child.wait().unwrap();
    assert_eq!(status.code().unwrap(), 0);

    let mut child = pty_process::blocking::Command::new("echo")
        .arg("bar")
        .spawn(&pty)
        .unwrap();

    let mut output = helpers::output(&pty);
    assert_eq!(output.next().unwrap(), "bar\r\n");

    let status = child.wait().unwrap();
    assert_eq!(status.code().unwrap(), 0);
}

#[cfg(feature = "async")]
#[test]
fn test_multiple_async() {
    use futures::stream::StreamExt as _;

    async_std::task::block_on(async {
        let pty = pty_process::Pty::new().unwrap();
        pty.resize(pty_process::Size::new(24, 80)).unwrap();

        let mut child = pty_process::Command::new("echo")
            .arg("foo")
            .spawn(&pty)
            .unwrap();

        let mut output = helpers::output_async(&pty);
        assert_eq!(output.next().await.unwrap(), "foo\r\n");

        let status = child.status().await.unwrap();
        assert_eq!(status.code().unwrap(), 0);

        let mut child = pty_process::Command::new("echo")
            .arg("bar")
            .spawn(&pty)
            .unwrap();

        let mut output = helpers::output_async(&pty);
        assert_eq!(output.next().await.unwrap(), "bar\r\n");

        let status = child.status().await.unwrap();
        assert_eq!(status.code().unwrap(), 0);
    });
}

#[test]
fn test_multiple_configured() {
    use std::io::BufRead as _;
    use std::os::unix::io::FromRawFd as _;

    let pty = pty_process::blocking::Pty::new().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();

    let (stderr_pipe_r, stderr_pipe_w) = nix::unistd::pipe().unwrap();
    let mut stderr_pipe_r = std::io::BufReader::new(unsafe {
        std::fs::File::from_raw_fd(stderr_pipe_r)
    });
    let (pre_exec_pipe_r, pre_exec_pipe_w) = nix::unistd::pipe().unwrap();
    let mut pre_exec_pipe_r = std::io::BufReader::new(unsafe {
        std::fs::File::from_raw_fd(pre_exec_pipe_r)
    });
    let mut cmd = pty_process::blocking::Command::new("perl");
    cmd.arg("-Esay 'foo'; say STDERR 'foo-stderr'; open my $fh, '>&=3'; say $fh 'foo-3';")
        .stderr(unsafe { std::process::Stdio::from_raw_fd(stderr_pipe_w) });
    unsafe {
        cmd.pre_exec(move || {
            nix::unistd::dup2(pre_exec_pipe_w, 3)?;
            nix::fcntl::fcntl(
                3,
                nix::fcntl::F_SETFD(nix::fcntl::FdFlag::empty()),
            )?;
            Ok(())
        });
    }
    let mut child = cmd.spawn(&pty).unwrap();

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

    let mut child = cmd.spawn(&pty).unwrap();

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
#[test]
fn test_multiple_configured_async() {
    use async_std::io::prelude::BufReadExt as _;
    use futures::stream::StreamExt as _;
    use std::os::unix::io::FromRawFd as _;

    async_std::task::block_on(async {
        let pty = pty_process::Pty::new().unwrap();
        pty.resize(pty_process::Size::new(24, 80)).unwrap();

        let (stderr_pipe_r, stderr_pipe_w) = nix::unistd::pipe().unwrap();
        let mut stderr_pipe_r = async_std::io::BufReader::new(unsafe {
            async_std::fs::File::from_raw_fd(stderr_pipe_r)
        });
        let (pre_exec_pipe_r, pre_exec_pipe_w) = nix::unistd::pipe().unwrap();
        let mut pre_exec_pipe_r = async_std::io::BufReader::new(unsafe {
            async_std::fs::File::from_raw_fd(pre_exec_pipe_r)
        });
        let mut cmd = pty_process::Command::new("perl");
        cmd.arg("-Esay 'foo'; say STDERR 'foo-stderr'; open my $fh, '>&=3'; say $fh 'foo-3';")
            .stderr(unsafe { std::process::Stdio::from_raw_fd(stderr_pipe_w) });
        unsafe {
            cmd.pre_exec(move || {
                nix::unistd::dup2(pre_exec_pipe_w, 3)?;
                nix::fcntl::fcntl(
                    3,
                    nix::fcntl::F_SETFD(nix::fcntl::FdFlag::empty()),
                )?;
                Ok(())
            });
        }
        let mut child = cmd.spawn(&pty).unwrap();

        let mut output = helpers::output_async(&pty);
        assert_eq!(output.next().await.unwrap(), "foo\r\n");

        let mut buf = vec![];
        async_std::future::timeout(
            std::time::Duration::from_secs(5),
            stderr_pipe_r.read_until(b'\n', &mut buf),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(
            std::string::String::from_utf8(buf).unwrap(),
            "foo-stderr\n"
        );

        let mut buf = vec![];
        async_std::future::timeout(
            std::time::Duration::from_secs(5),
            pre_exec_pipe_r.read_until(b'\n', &mut buf),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(std::string::String::from_utf8(buf).unwrap(), "foo-3\n");

        let status = child.status().await.unwrap();
        assert_eq!(status.code().unwrap(), 0);

        let mut child = cmd.spawn(&pty).unwrap();

        assert_eq!(output.next().await.unwrap(), "foo\r\n");

        let mut buf = vec![];
        async_std::future::timeout(
            std::time::Duration::from_secs(5),
            stderr_pipe_r.read_until(b'\n', &mut buf),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(
            std::string::String::from_utf8(buf).unwrap(),
            "foo-stderr\n"
        );

        let mut buf = vec![];
        async_std::future::timeout(
            std::time::Duration::from_secs(5),
            pre_exec_pipe_r.read_until(b'\n', &mut buf),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(std::string::String::from_utf8(buf).unwrap(), "foo-3\n");

        let status = child.status().await.unwrap();
        assert_eq!(status.code().unwrap(), 0);
    });
}

#[test]
fn test_controlling_terminal() {
    let pty = pty_process::blocking::Pty::new().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();
    let mut child = pty_process::blocking::Command::new("perl")
        .arg("-Eopen my $fh, '<', '/dev/tty' or die; if (-t $fh) { say 'true' } else { say 'false' }")
        .spawn(&pty)
        .unwrap();

    let mut output = helpers::output(&pty);
    assert_eq!(output.next().unwrap(), "true\r\n");

    let status = child.wait().unwrap();
    assert_eq!(status.code().unwrap(), 0);
}

#[cfg(feature = "async")]
#[test]
fn test_controlling_terminal_async() {
    use futures::stream::StreamExt as _;

    async_std::task::block_on(async {
        let pty = pty_process::Pty::new().unwrap();
        pty.resize(pty_process::Size::new(24, 80)).unwrap();
        let mut child = pty_process::Command::new("perl")
            .arg("-Eopen my $fh, '<', '/dev/tty' or die; if (-t $fh) { say 'true' } else { say 'false' }")
            .spawn(&pty)
            .unwrap();

        let mut output = helpers::output_async(&pty);
        assert_eq!(output.next().await.unwrap(), "true\r\n");

        let status = child.status().await.unwrap();
        assert_eq!(status.code().unwrap(), 0);
    });
}

#[test]
fn test_session_leader() {
    let pty = pty_process::blocking::Pty::new().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();
    let mut child = pty_process::blocking::Command::new("python")
        .arg("-cimport os; print(os.getpid() == os.getsid(0))")
        .spawn(&pty)
        .unwrap();

    let mut output = helpers::output(&pty);
    assert_eq!(output.next().unwrap(), "True\r\n");

    let status = child.wait().unwrap();
    assert_eq!(status.code().unwrap(), 0);
}

#[cfg(feature = "async")]
#[test]
fn test_session_leader_async() {
    use futures::stream::StreamExt as _;

    async_std::task::block_on(async {
        let pty = pty_process::Pty::new().unwrap();
        pty.resize(pty_process::Size::new(24, 80)).unwrap();
        let mut child = pty_process::Command::new("python")
            .arg("-cimport os; print(os.getpid() == os.getsid(0))")
            .spawn(&pty)
            .unwrap();

        {
            let mut output = helpers::output_async(&pty);
            assert_eq!(output.next().await.unwrap(), "True\r\n");
        }

        let status = child.status().await.unwrap();
        assert_eq!(status.code().unwrap(), 0);
    });
}
