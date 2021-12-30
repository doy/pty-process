#[test]
fn test_winch_std() {
    use std::io::{Read as _, Write as _};

    let mut pty = pty_process::blocking::Pty::new().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();
    let mut child = pty_process::blocking::Command::new("perl")
        .args(&[
            "-E",
            "$|++; $SIG{WINCH} = sub { say 'WINCH' }; say 'started'; <>",
        ])
        .spawn(&pty)
        .unwrap();

    let mut buf = [0u8; 1024];
    let bytes = pty.read(&mut buf).unwrap();
    assert_eq!(&buf[..bytes], b"started\r\n");

    pty.resize(pty_process::Size::new(25, 80)).unwrap();

    let bytes = pty.read(&mut buf).unwrap();
    assert_eq!(&buf[..bytes], b"WINCH\r\n");

    pty.write_all(b"\n").unwrap();
    let status = child.wait().unwrap();
    assert_eq!(status.code().unwrap(), 0);
}

#[cfg(feature = "async")]
#[test]
fn test_winch_async() {
    use async_std::io::prelude::WriteExt as _;
    use async_std::io::ReadExt as _;

    let status = async_std::task::block_on(async {
        let mut pty = pty_process::Pty::new().unwrap();
        pty.resize(pty_process::Size::new(24, 80)).unwrap();
        let mut child = pty_process::Command::new("perl")
            .args(&[
                "-E",
                "$|++; $SIG{WINCH} = sub { say 'WINCH' }; say 'started'; <>",
            ])
            .spawn(&pty)
            .unwrap();

        let mut buf = [0u8; 1024];
        let bytes = pty.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..bytes], b"started\r\n");

        pty.resize(pty_process::Size::new(25, 80)).unwrap();

        let bytes = pty.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..bytes], b"WINCH\r\n");

        pty.write_all(b"\n").await.unwrap();
        child.status().await.unwrap()
    });
    assert_eq!(status.code().unwrap(), 0);
}
