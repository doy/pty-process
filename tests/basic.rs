mod helpers;

#[test]
fn test_cat_blocking() {
    use std::io::Write as _;

    let mut pty = pty_process::blocking::Pty::new().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();
    let mut child = pty_process::blocking::Command::new("cat")
        .spawn(&pty.pts().unwrap())
        .unwrap();

    pty.write_all(b"foo\n").unwrap();

    let mut output = helpers::output(&pty);
    assert_eq!(output.next().unwrap(), "foo\r\n");
    assert_eq!(output.next().unwrap(), "foo\r\n");

    pty.write_all(&[4u8]).unwrap();
    let status = child.wait().unwrap();
    assert_eq!(status.code().unwrap(), 0);
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_cat_async() {
    use futures::stream::StreamExt as _;
    use tokio::io::AsyncWriteExt as _;

    let mut pty = pty_process::Pty::new().unwrap();
    let pts = pty.pts().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();
    let mut child = pty_process::Command::new("cat").spawn(&pts).unwrap();

    let (pty_r, mut pty_w) = pty.split();

    pty_w.write_all(b"foo\n").await.unwrap();

    let mut output = helpers::output_async(pty_r);
    assert_eq!(output.next().await.unwrap(), "foo\r\n");
    assert_eq!(output.next().await.unwrap(), "foo\r\n");

    pty_w.write_all(&[4u8]).await.unwrap();
    let status = child.wait().await.unwrap();
    assert_eq!(status.code().unwrap(), 0);
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_yes_async() {
    use tokio::io::AsyncReadExt as _;

    let mut pty = pty_process::Pty::new().unwrap();
    let pts = pty.pts().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();
    let mut child = pty_process::Command::new("yes").spawn(&pts).unwrap();

    let mut buf = [0u8; 3];

    let bytes = pty.read_buf(&mut &mut buf[..]).await.unwrap();
    assert_eq!(&buf[..bytes], b"y\r\n");

    let (mut pty_r, _pty_w) = pty.split();
    let bytes = pty_r.read_buf(&mut &mut buf[..]).await.unwrap();
    assert_eq!(&buf[..bytes], b"y\r\n");

    let (mut pty_r, _pty_w) = pty.into_split();
    let bytes = pty_r.read_buf(&mut &mut buf[..]).await.unwrap();
    assert_eq!(&buf[..bytes], b"y\r\n");

    child.kill().await.unwrap()
}
