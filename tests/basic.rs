mod helpers;

#[test]
fn test_cat_blocking() {
    use std::io::Write as _;

    let pty = pty_process::blocking::Pty::new().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();
    let mut child = pty_process::blocking::Command::new("cat")
        .spawn(&pty)
        .unwrap();

    (&pty).write_all(b"foo\n").unwrap();

    let mut output = helpers::output(&pty);
    assert_eq!(output.next().unwrap(), "foo\r\n");
    assert_eq!(output.next().unwrap(), "foo\r\n");

    (&pty).write_all(&[4u8]).unwrap();
    let status = child.wait().unwrap();
    assert_eq!(status.code().unwrap(), 0);
}

#[cfg(feature = "async")]
#[test]
fn test_cat_async_std() {
    use async_std::io::prelude::WriteExt as _;
    use futures::stream::StreamExt as _;

    let status = async_std::task::block_on(async {
        let pty = pty_process::Pty::new().unwrap();
        pty.resize(pty_process::Size::new(24, 80)).unwrap();
        let mut child = pty_process::Command::new("cat").spawn(&pty).unwrap();

        (&pty).write_all(b"foo\n").await.unwrap();

        let mut output = helpers::output_async(&pty);
        assert_eq!(output.next().await.unwrap(), "foo\r\n");
        assert_eq!(output.next().await.unwrap(), "foo\r\n");

        (&pty).write_all(&[4u8]).await.unwrap();
        child.status().await.unwrap()
    });
    assert_eq!(status.code().unwrap(), 0);
}
