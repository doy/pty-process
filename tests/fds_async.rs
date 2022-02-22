mod helpers;

#[cfg(feature = "async")]
#[test]
fn test_fds_async() {
    use futures::stream::StreamExt as _;

    check_open_fds(&[0, 1, 2]);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    // run once to ensure all of the fds in the tokio machinery are
    // allocated
    rt.block_on(async {
        let mut pty = pty_process::Pty::new().unwrap();
        let pts = pty.pts().unwrap();
        pty.resize(pty_process::Size::new(24, 80)).unwrap();
        let mut child = pty_process::Command::new("perl")
            .arg(
                "-Efor my $fd (0..255) { \
                open my $fh, \"<&=$fd\"; \
                print $fd if stat $fh \
                }; \
                say",
            )
            .spawn(&pts)
            .unwrap();

        let (pty_r, _) = pty.split();
        let mut output = helpers::output_async(pty_r);
        assert_eq!(output.next().await.unwrap(), "012\r\n");

        let status = child.wait().await.unwrap();
        assert_eq!(status.code().unwrap(), 0);
    });

    rt.block_on(async {
        let fds = get_open_fds();

        let mut pty = pty_process::Pty::new().unwrap();
        let pts = pty.pts().unwrap();
        pty.resize(pty_process::Size::new(24, 80)).unwrap();
        let mut child = pty_process::Command::new("perl")
            .arg(
                "-Efor my $fd (0..255) { \
                open my $fh, \"<&=$fd\"; \
                print $fd if stat $fh \
                }; \
                say",
            )
            .spawn(&pts)
            .unwrap();

        let (pty_r, _) = pty.split();
        let mut output = helpers::output_async(pty_r);
        assert_eq!(output.next().await.unwrap(), "012\r\n");

        let status = child.wait().await.unwrap();
        assert_eq!(status.code().unwrap(), 0);
        drop(output);
        drop(pts);
        drop(pty);

        check_open_fds(&fds);
    });

    rt.block_on(async {
        let fds = get_open_fds();

        let mut pty = pty_process::Pty::new().unwrap();
        let pts = pty.pts().unwrap();
        pty.resize(pty_process::Size::new(24, 80)).unwrap();
        let mut child = pty_process::Command::new("perl")
            .arg("-Efor my $fd (0..255) { open my $fh, \"<&=$fd\"; print $fd if stat $fh }; say")
            .stderr(std::process::Stdio::null())
            .spawn(&pts)
            .unwrap();

        let (pty_r, _) = pty.split();
        let mut output = helpers::output_async(pty_r);
        assert_eq!(output.next().await.unwrap(), "012\r\n");

        let status = child.wait().await.unwrap();
        assert_eq!(status.code().unwrap(), 0);
        drop(output);
        drop(pts);
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
