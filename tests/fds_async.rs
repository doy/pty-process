mod helpers;

#[cfg(feature = "async")]
#[test]
fn test_fds_async() {
    use futures::stream::StreamExt as _;

    check_open_fds(&[0, 1, 2]);

    // run once to ensure all of the fds in the async_std machinery are
    // allocated
    async_std::task::block_on(async {
        let pty = pty_process::Pty::new().unwrap();
        pty.resize(pty_process::Size::new(24, 80)).unwrap();
        let mut child = pty_process::Command::new("perl")
            .arg("-Efor my $fd (0..255) { open my $fh, \"<&=$fd\"; print $fd if stat $fh }; say")
            .spawn(&pty)
            .unwrap();

        let mut output = helpers::output_async(&pty);
        assert_eq!(output.next().await.unwrap(), "012\r\n");

        let status = child.status().await.unwrap();
        assert_eq!(status.code().unwrap(), 0);
    });

    async_std::task::block_on(async {
        let fds = get_open_fds();

        let pty = pty_process::Pty::new().unwrap();
        pty.resize(pty_process::Size::new(24, 80)).unwrap();
        let mut child = pty_process::Command::new("perl")
            .arg("-Efor my $fd (0..255) { open my $fh, \"<&=$fd\"; print $fd if stat $fh }; say")
            .spawn(&pty)
            .unwrap();

        let mut output = helpers::output_async(&pty);
        assert_eq!(output.next().await.unwrap(), "012\r\n");

        let status = child.status().await.unwrap();
        assert_eq!(status.code().unwrap(), 0);
        drop(output);
        drop(pty);

        check_open_fds(&fds);
    });

    async_std::task::block_on(async {
        let fds = get_open_fds();

        let pty = pty_process::Pty::new().unwrap();
        pty.resize(pty_process::Size::new(24, 80)).unwrap();
        let mut child = pty_process::Command::new("perl")
            .arg("-Efor my $fd (0..255) { open my $fh, \"<&=$fd\"; print $fd if stat $fh }; say")
            .stderr(std::process::Stdio::null())
            .spawn(&pty)
            .unwrap();

        let mut output = helpers::output_async(&pty);
        assert_eq!(output.next().await.unwrap(), "012\r\n");

        let status = child.status().await.unwrap();
        assert_eq!(status.code().unwrap(), 0);
        drop(output);
        drop(pty);

        check_open_fds(&fds);
    });
}

#[cfg(feature = "async")]
#[track_caller]
fn check_open_fds(expected: &[i32]) {
    assert_eq!(get_open_fds(), expected);
}

#[cfg(feature = "async")]
fn get_open_fds() -> Vec<i32> {
    (0..=255)
        .filter(|fd| nix::sys::stat::fstat(*fd).is_ok())
        .collect()
}
