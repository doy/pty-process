use pty_process::Command as _;

#[cfg(feature = "backend-std")]
#[test]
fn test_cat_std() {
    use std::io::{Read as _, Write as _};

    let (mut child, mut pty) = std::process::Command::new("cat")
        .spawn_pty(Some(&pty_process::Size::new(24, 80)))
        .unwrap();

    pty.write_all(b"foo\n").unwrap();
    // the pty will echo the written bytes back immediately, but the
    // subprocess needs to generate its own output, which takes time, so we
    // can't just read immediately (we may just get the echoed bytes). because
    // the output generation is happening in the subprocess, we also don't
    // have any way to know when (or if!) the subprocess will decide to send
    // its output, so sleeping is the best we can do.
    std::thread::sleep(std::time::Duration::from_secs(1));

    let mut buf = [0u8; 1024];
    let bytes = pty.read(&mut buf).unwrap();
    assert_eq!(&buf[..bytes], b"foo\r\nfoo\r\n");

    pty.write_all(&[4u8]).unwrap();
    let status = child.wait().unwrap();
    assert_eq!(status.code().unwrap(), 0);
}

#[cfg(feature = "backend-smol")]
#[test]
fn test_cat_smol() {
    use smol::io::{AsyncReadExt as _, AsyncWriteExt as _};

    let status = smol::block_on(async {
        let mut child = smol::process::Command::new("cat")
            .spawn_pty(Some(&pty_process::Size::new(24, 80)))
            .unwrap();

        child.pty().write_all(b"foo\n").await.unwrap();
        // the pty will echo the written bytes back immediately, but the
        // subprocess needs to generate its own output, which takes time, so
        // we can't just read immediately (we may just get the echoed bytes).
        // because the output generation is happening in the subprocess, we
        // also don't have any way to know when (or if!) the subprocess will
        // decide to send its output, so sleeping is the best we can do.
        smol::Timer::after(std::time::Duration::from_secs(1)).await;

        let mut buf = [0u8; 1024];
        let bytes = child.pty().read(&mut buf).await.unwrap();
        assert_eq!(&buf[..bytes], b"foo\r\nfoo\r\n");

        child.pty().write_all(&[4u8]).await.unwrap();
        child.status().await.unwrap()
    });
    assert_eq!(status.code().unwrap(), 0);
}

#[cfg(feature = "backend-async-std")]
#[test]
fn test_cat_async_std() {
    use async_std::io::prelude::WriteExt as _;
    use async_std::io::ReadExt as _;

    let status = async_std::task::block_on(async {
        let mut child = async_std::process::Command::new("cat")
            .spawn_pty(Some(&pty_process::Size::new(24, 80)))
            .unwrap();

        child.pty().write_all(b"foo\n").await.unwrap();
        // the pty will echo the written bytes back immediately, but the
        // subprocess needs to generate its own output, which takes time, so
        // we can't just read immediately (we may just get the echoed bytes).
        // because the output generation is happening in the subprocess, we
        // also don't have any way to know when (or if!) the subprocess will
        // decide to send its output, so sleeping is the best we can do.
        async_std::task::sleep(std::time::Duration::from_secs(1)).await;

        let mut buf = [0u8; 1024];
        let bytes = child.pty().read(&mut buf).await.unwrap();
        assert_eq!(&buf[..bytes], b"foo\r\nfoo\r\n");

        child.pty().write_all(&[4u8]).await.unwrap();
        child.status().await.unwrap()
    });
    assert_eq!(status.code().unwrap(), 0);
}

#[cfg(feature = "backend-tokio")]
#[test]
fn test_cat_tokio() {
    use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

    async fn async_test_cat_tokio() {
        let mut child = tokio::process::Command::new("cat")
            .spawn_pty(Some(&pty_process::Size::new(24, 80)))
            .unwrap();

        child.pty_mut().write_all(b"foo\n").await.unwrap();
        // the pty will echo the written bytes back immediately, but the
        // subprocess needs to generate its own output, which takes time, so
        // we can't just read immediately (we may just get the echoed bytes).
        // because the output generation is happening in the subprocess, we
        // also don't have any way to know when (or if!) the subprocess will
        // decide to send its output, so sleeping is the best we can do.
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let mut buf = [0u8; 1024];
        let bytes = child.pty_mut().read(&mut buf).await.unwrap();
        assert_eq!(&buf[..bytes], b"foo\r\nfoo\r\n");

        child.pty_mut().write_all(&[4u8]).await.unwrap();

        let status = child.wait().await.unwrap();
        assert_eq!(status.code().unwrap(), 0);
    }
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        async_test_cat_tokio().await;
    });
}
