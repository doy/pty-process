mod helpers;

#[cfg(feature = "async")]
#[tokio::test]
async fn test_split() {
    use futures::stream::StreamExt as _;
    use tokio::io::AsyncWriteExt as _;

    let mut pty = pty_process::Pty::new().unwrap();
    let pts = pty.pts().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();
    let mut cmd = pty_process::Command::new("perl");
    cmd.args(["-plE", "BEGIN { $SIG{WINCH} = sub { say 'WINCH' } }"]);
    let mut child = cmd.spawn(&pts).unwrap();

    {
        pty.write_all(b"foo\n").await.unwrap();
        let (pty_r, _) = pty.split();
        let mut output = helpers::output_async(pty_r);
        assert_eq!(output.next().await.unwrap(), "foo\r\n");
        assert_eq!(output.next().await.unwrap(), "foo\r\n");
    }

    {
        let (pty_r, mut pty_w) = pty.split();
        pty_w.write_all(b"foo\n").await.unwrap();
        let mut output = helpers::output_async(pty_r);
        assert_eq!(output.next().await.unwrap(), "foo\r\n");
        assert_eq!(output.next().await.unwrap(), "foo\r\n");
    }

    {
        let (pty_r, pty_w) = pty.split();
        pty_w.resize(pty_process::Size::new(25, 80)).unwrap();
        let mut output = helpers::output_async(pty_r);
        assert_eq!(output.next().await.unwrap(), "WINCH\r\n");
    }

    pty.write_all(&[4u8]).await.unwrap();
    child.wait().await.unwrap();
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_into_split() {
    use tokio::io::{AsyncBufReadExt as _, AsyncWriteExt as _};

    let mut pty = pty_process::Pty::new().unwrap();
    let pts = pty.pts().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();
    let mut cmd = pty_process::Command::new("perl");
    cmd.args(["-plE", "BEGIN { $SIG{WINCH} = sub { say 'WINCH' } }"]);
    let mut child = cmd.spawn(&pts).unwrap();

    {
        pty.write_all(b"foo\n").await.unwrap();
        let (pty_r, pty_w) = pty.into_split();
        let mut ptybuf = tokio::io::BufReader::new(pty_r);
        for _ in 0..2 {
            let mut buf = vec![];
            tokio::time::timeout(
                std::time::Duration::from_secs(5),
                ptybuf.read_until(b'\n', &mut buf),
            )
            .await
            .unwrap()
            .unwrap();
            assert_eq!(&buf[..], b"foo\r\n");
        }
        pty = ptybuf.into_inner().unsplit(pty_w).unwrap();
    }

    {
        let (pty_r, mut pty_w) = pty.into_split();
        pty_w.write_all(b"foo\n").await.unwrap();
        let mut ptybuf = tokio::io::BufReader::new(pty_r);
        for _ in 0..2 {
            let mut buf = vec![];
            tokio::time::timeout(
                std::time::Duration::from_secs(5),
                ptybuf.read_until(b'\n', &mut buf),
            )
            .await
            .unwrap()
            .unwrap();
            assert_eq!(&buf[..], b"foo\r\n");
        }
        pty = ptybuf.into_inner().unsplit(pty_w).unwrap();
    }

    {
        let (pty_r, pty_w) = pty.into_split();
        pty_w.resize(pty_process::Size::new(25, 80)).unwrap();
        let mut ptybuf = tokio::io::BufReader::new(pty_r);
        let mut buf = vec![];
        tokio::time::timeout(
            std::time::Duration::from_secs(5),
            ptybuf.read_until(b'\n', &mut buf),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(&buf[..], b"WINCH\r\n");
        pty = ptybuf.into_inner().unsplit(pty_w).unwrap();
    }

    pty.write_all(&[4u8]).await.unwrap();
    child.wait().await.unwrap();
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_into_split_error() {
    let pty1 = pty_process::Pty::new().unwrap();
    let pty2 = pty_process::Pty::new().unwrap();

    let (pty1_r, pty1_w) = pty1.into_split();
    let (pty2_r, pty2_w) = pty2.into_split();

    let (pty1_r, pty2_w) = if let Err(pty_process::Error::Unsplit(r, w)) =
        pty1_r.unsplit(pty2_w)
    {
        (r, w)
    } else {
        panic!("fail");
    };
    let (pty2_r, pty1_w) = if let Err(pty_process::Error::Unsplit(r, w)) =
        pty2_r.unsplit(pty1_w)
    {
        (r, w)
    } else {
        panic!("fail");
    };

    let _pty1 = pty1_r.unsplit(pty1_w).unwrap();
    let _pty2 = pty2_r.unsplit(pty2_w).unwrap();
}
