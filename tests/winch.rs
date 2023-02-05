mod helpers;

#[test]
fn test_winch_std() {
    use std::io::Write as _;

    let mut pty = pty_process::blocking::Pty::new().unwrap();
    let pts = pty.pts().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();
    let mut child = pty_process::blocking::Command::new("perl")
        .args([
            "-E",
            "$|++; $SIG{WINCH} = sub { say 'WINCH' }; say 'started'; <>",
        ])
        .spawn(&pts)
        .unwrap();

    let mut output = helpers::output(&pty);
    assert_eq!(output.next().unwrap(), "started\r\n");

    pty.resize(pty_process::Size::new(25, 80)).unwrap();
    assert_eq!(output.next().unwrap(), "WINCH\r\n");

    pty.write_all(b"\n").unwrap();
    let status = child.wait().unwrap();
    assert_eq!(status.code().unwrap(), 0);
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_winch_async() {
    use futures::stream::StreamExt as _;
    use tokio::io::AsyncWriteExt as _;

    let mut pty = pty_process::Pty::new().unwrap();
    let pts = pty.pts().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();
    let mut child = pty_process::Command::new("perl")
        .args(&[
            "-E",
            "$|++; $SIG{WINCH} = sub { say 'WINCH' }; say 'started'; <>",
        ])
        .spawn(&pts)
        .unwrap();

    let (pty_r, mut pty_w) = pty.split();
    let mut output = helpers::output_async(pty_r);
    assert_eq!(output.next().await.unwrap(), "started\r\n");

    pty_w.resize(pty_process::Size::new(25, 80)).unwrap();
    assert_eq!(output.next().await.unwrap(), "WINCH\r\n");

    pty_w.write_all(b"\n").await.unwrap();
    let status = child.wait().await.unwrap();
    assert_eq!(status.code().unwrap(), 0);
}
