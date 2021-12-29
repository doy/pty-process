#[test]
fn test_winch_std() {
    use std::io::{Read as _, Write as _};

    let pty = pty_process::blocking::Pty::new().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();
    let mut child = pty_process::blocking::Command::new("perl")
        .args(&[
            "-E",
            "$|++; $SIG{WINCH} = sub { say 'WINCH' }; say 'started'; <>",
        ])
        .spawn(pty)
        .unwrap();

    let mut buf = [0u8; 1024];
    let bytes = child.pty().read(&mut buf).unwrap();
    assert_eq!(&buf[..bytes], b"started\r\n");

    child.pty().resize(pty_process::Size::new(25, 80)).unwrap();

    let bytes = child.pty().read(&mut buf).unwrap();
    assert_eq!(&buf[..bytes], b"WINCH\r\n");

    child.pty().write_all(b"\n").unwrap();
    let status = child.wait().unwrap();
    assert_eq!(status.code().unwrap(), 0);
}

#[cfg(feature = "async")]
#[test]
fn test_winch_async_std() {
    use async_std::io::prelude::WriteExt as _;
    use async_std::io::ReadExt as _;

    let status = async_std::task::block_on(async {
        let pty = pty_process::Pty::new().unwrap();
        pty.resize(pty_process::Size::new(24, 80)).unwrap();
        let mut child = pty_process::Command::new("perl")
            .args(&[
                "-E",
                "$|++; $SIG{WINCH} = sub { say 'WINCH' }; say 'started'; <>",
            ])
            .spawn(pty)
            .unwrap();

        let mut buf = [0u8; 1024];
        let bytes = child.pty().read(&mut buf).await.unwrap();
        assert_eq!(&buf[..bytes], b"started\r\n");

        child.pty().resize(pty_process::Size::new(25, 80)).unwrap();

        let bytes = child.pty().read(&mut buf).await.unwrap();
        assert_eq!(&buf[..bytes], b"WINCH\r\n");

        child.pty().write_all(b"\n").await.unwrap();
        child.status().await.unwrap()
    });
    assert_eq!(status.code().unwrap(), 0);
}

#[cfg(feature = "async")]
#[test]
fn test_winch_smol() {
    use smol::io::{AsyncReadExt as _, AsyncWriteExt as _};

    let status = smol::block_on(async {
        let pty = pty_process::Pty::new().unwrap();
        pty.resize(pty_process::Size::new(24, 80)).unwrap();
        let mut child = pty_process::Command::new("perl")
            .args(&[
                "-E",
                "$|++; $SIG{WINCH} = sub { say 'WINCH' }; say 'started'; <>",
            ])
            .spawn(pty)
            .unwrap();

        let mut buf = [0u8; 1024];
        let bytes = child.pty().read(&mut buf).await.unwrap();
        assert_eq!(&buf[..bytes], b"started\r\n");

        child.pty().resize(pty_process::Size::new(25, 80)).unwrap();

        let bytes = child.pty().read(&mut buf).await.unwrap();
        assert_eq!(&buf[..bytes], b"WINCH\r\n");

        child.pty().write_all(b"\n").await.unwrap();
        child.status().await.unwrap()
    });
    assert_eq!(status.code().unwrap(), 0);
}

#[cfg(feature = "async")]
#[test]
fn test_winch_tokio() {
    use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
    use tokio_util::compat::FuturesAsyncReadCompatExt as _;

    async fn async_test_cat_tokio() -> std::process::ExitStatus {
        let pty = pty_process::Pty::new().unwrap();
        pty.resize(pty_process::Size::new(24, 80)).unwrap();
        let mut child = pty_process::Command::new("perl")
            .args(&[
                "-E",
                "$|++; $SIG{WINCH} = sub { say 'WINCH' }; say 'started'; <>",
            ])
            .spawn(pty)
            .unwrap();

        let mut buf = [0u8; 1024];
        let bytes = child.pty().compat().read(&mut buf).await.unwrap();
        assert_eq!(&buf[..bytes], b"started\r\n");

        child.pty().resize(pty_process::Size::new(25, 80)).unwrap();

        let bytes = child.pty().compat().read(&mut buf).await.unwrap();
        assert_eq!(&buf[..bytes], b"WINCH\r\n");

        child.pty().compat().write_all(b"\n").await.unwrap();
        child.status().await.unwrap()
    }
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let status = async_test_cat_tokio().await;
        assert_eq!(status.code().unwrap(), 0);
    });
}
