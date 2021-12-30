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
