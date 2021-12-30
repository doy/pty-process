mod helpers;

#[test]
fn test_winch_std() {
    use std::io::Write as _;

    let pty = pty_process::blocking::Pty::new().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();
    let mut child = pty_process::blocking::Command::new("perl")
        .args(&[
            "-E",
            "$|++; $SIG{WINCH} = sub { say 'WINCH' }; say 'started'; <>",
        ])
        .spawn(&pty)
        .unwrap();

    let mut output = helpers::output(&pty);
    assert_eq!(output.next().unwrap(), "started\r\n");

    pty.resize(pty_process::Size::new(25, 80)).unwrap();
    assert_eq!(output.next().unwrap(), "WINCH\r\n");

    (&pty).write_all(b"\n").unwrap();
    let status = child.wait().unwrap();
    assert_eq!(status.code().unwrap(), 0);
}

#[cfg(feature = "async")]
#[test]
fn test_winch_async() {
    use async_std::io::prelude::WriteExt as _;
    use futures::stream::StreamExt as _;

    let status = async_std::task::block_on(async {
        let pty = pty_process::Pty::new().unwrap();
        pty.resize(pty_process::Size::new(24, 80)).unwrap();
        let mut child = pty_process::Command::new("perl")
            .args(&[
                "-E",
                "$|++; $SIG{WINCH} = sub { say 'WINCH' }; say 'started'; <>",
            ])
            .spawn(&pty)
            .unwrap();

        let mut output = helpers::output_async(&pty);
        assert_eq!(output.next().await.unwrap(), "started\r\n");

        pty.resize(pty_process::Size::new(25, 80)).unwrap();
        assert_eq!(output.next().await.unwrap(), "WINCH\r\n");

        (&pty).write_all(b"\n").await.unwrap();
        child.status().await.unwrap()
    });
    assert_eq!(status.code().unwrap(), 0);
}
