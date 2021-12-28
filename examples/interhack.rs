mod raw_guard;

#[cfg(feature = "backend-smol")]
mod main {
    use smol::io::{AsyncReadExt as _, AsyncWriteExt as _};

    pub async fn run(
        child: &mut pty_process::smol::Child,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let _raw = super::raw_guard::RawGuard::new();

        let ex = smol::Executor::new();

        let input = ex.spawn(async {
            let mut buf = [0_u8; 4096];
            let mut stdin = smol::Unblock::new(std::io::stdin());
            loop {
                match stdin.read(&mut buf).await {
                    Ok(bytes) => {
                        // engrave Elbereth with ^E
                        if buf[..bytes].contains(&5u8) {
                            for byte in buf[..bytes].iter() {
                                match byte {
                                    5u8 => child
                                        .pty()
                                        .write_all(b"E-  Elbereth\n")
                                        .await
                                        .unwrap(),
                                    _ => child
                                        .pty()
                                        .write_all(&[*byte])
                                        .await
                                        .unwrap(),
                                }
                            }
                        } else {
                            child
                                .pty()
                                .write_all(&buf[..bytes])
                                .await
                                .unwrap();
                        }
                    }
                    Err(e) => {
                        eprintln!("stdin read failed: {:?}", e);
                        break;
                    }
                }
            }
        });
        let output = ex.spawn(async {
            let mut buf = [0_u8; 4096];
            let mut stdout = smol::Unblock::new(std::io::stdout());
            #[allow(clippy::trivial_regex)]
            let re = regex::bytes::Regex::new("Elbereth").unwrap();
            loop {
                match child.pty().read(&mut buf).await {
                    Ok(bytes) => {
                        // highlight successful Elbereths
                        if re.is_match(&buf[..bytes]) {
                            stdout
                                .write_all(&re.replace_all(
                                    &buf[..bytes],
                                    &b"\x1b[35m$0\x1b[m"[..],
                                ))
                                .await
                                .unwrap();
                        } else {
                            stdout.write_all(&buf[..bytes]).await.unwrap();
                        }
                        stdout.flush().await.unwrap();
                    }
                    Err(e) => {
                        // EIO means that the process closed the other
                        // end of the pty
                        if e.raw_os_error() != Some(libc::EIO) {
                            eprintln!("pty read failed: {:?}", e);
                        }
                        break;
                    }
                }
            }
        });

        let wait = async {
            child.status_no_drop().await.unwrap();
        };

        ex.run(smol::future::or(smol::future::or(input, output), wait))
            .await;

        Ok(())
    }
}

#[cfg(feature = "backend-smol")]
fn main() {
    use std::os::unix::process::ExitStatusExt as _;

    let (w, h) = if let Some((w, h)) = term_size::dimensions() {
        (w as u16, h as u16)
    } else {
        (80, 24)
    };
    let status = smol::block_on(async {
        let pty = pty_process::smol::Pty::new().unwrap();
        pty.resize(pty_process::Size::new(h, w)).unwrap();
        let mut child = pty_process::smol::Command::new("nethack")
            .spawn(pty)
            .unwrap();
        main::run(&mut child).await.unwrap();
        child.status().await.unwrap()
    });
    std::process::exit(
        status
            .code()
            .unwrap_or_else(|| status.signal().unwrap_or(0) + 128),
    );
}

#[cfg(not(feature = "backend-smol"))]
fn main() {
    unimplemented!()
}
