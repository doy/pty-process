mod helpers;

#[cfg(feature = "async")]
#[test]
fn test_fds_async() {
    use futures::stream::StreamExt as _;

    let mut expected = String::new();
    for fd in get_open_fds() {
        expected.push_str(&format!("{fd}"));
    }

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    // run once to ensure all of the fds in the tokio machinery are
    // allocated
    rt.block_on(async {
        let (mut pty, pts) = pty_process::open().unwrap();
        pty.resize(pty_process::Size::new(24, 80)).unwrap();
        let mut child = pty_process::Command::new("perl")
            .arg(
                "-Efor my $fd (0..255) { \
                open my $fh, \"<&=$fd\"; \
                print $fd if stat $fh \
                }; \
                say",
            )
            .spawn(pts)
            .unwrap();

        let (pty_r, _) = pty.split();
        let mut output = helpers::output_async(pty_r);
        assert_eq!(output.next().await.unwrap(), format!("{expected}\r\n"));

        let status = child.wait().await.unwrap();
        assert_eq!(status.code().unwrap(), 0);
    });

    rt.block_on(async {
        let fds = get_open_fds();

        let (mut pty, pts) = pty_process::open().unwrap();
        pty.resize(pty_process::Size::new(24, 80)).unwrap();
        let mut child = pty_process::Command::new("perl")
            .arg(
                "-Efor my $fd (0..255) { \
                open my $fh, \"<&=$fd\"; \
                print $fd if stat $fh \
                }; \
                say",
            )
            .spawn(pts)
            .unwrap();

        let (pty_r, _) = pty.split();
        let mut output = helpers::output_async(pty_r);
        assert_eq!(output.next().await.unwrap(), format!("{expected}\r\n"));

        let status = child.wait().await.unwrap();
        assert_eq!(status.code().unwrap(), 0);
        drop(output);
        drop(pty);

        check_open_fds(&fds);
    });

    rt.block_on(async {
        let fds = get_open_fds();

        let (mut pty, pts) = pty_process::open().unwrap();
        pty.resize(pty_process::Size::new(24, 80)).unwrap();
        let mut child = pty_process::Command::new("perl")
            .arg("-Efor my $fd (0..255) { open my $fh, \"<&=$fd\"; print $fd if stat $fh }; say")
            .stderr(std::process::Stdio::null())
            .spawn(pts)
            .unwrap();

        let (pty_r, _) = pty.split();
        let mut output = helpers::output_async(pty_r);
        assert_eq!(output.next().await.unwrap(), format!("{expected}\r\n"));

        let status = child.wait().await.unwrap();
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
