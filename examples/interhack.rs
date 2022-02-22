mod raw_guard;

#[cfg(feature = "async")]
mod main {
    use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

    pub async fn run(
        child: &mut tokio::process::Child,
        pty: &mut pty_process::Pty,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let _raw = super::raw_guard::RawGuard::new();

        let mut in_buf = [0_u8; 4096];
        let mut out_buf = [0_u8; 4096];

        let mut stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();

        #[allow(clippy::trivial_regex)]
        let re = regex::bytes::Regex::new("Elbereth").unwrap();

        loop {
            tokio::select! {
                bytes = stdin.read(&mut in_buf) => match bytes {
                    Ok(bytes) => {
                        // engrave Elbereth with ^E
                        if in_buf[..bytes].contains(&5u8) {
                            for byte in in_buf[..bytes].iter() {
                                match byte {
                                    5u8 => pty
                                        .write_all(b"E-  Elbereth\n")
                                        .await
                                        .unwrap(),
                                    _ => pty
                                        .write_all(&[*byte])
                                        .await
                                        .unwrap(),
                                }
                            }
                        } else {
                            pty.write_all(&in_buf[..bytes]).await.unwrap();
                        }
                    }
                    Err(e) => {
                        eprintln!("stdin read failed: {:?}", e);
                        break;
                    }
                },
                bytes = pty.read(&mut out_buf) => match bytes {
                    Ok(bytes) => {
                        // highlight successful Elbereths
                        if re.is_match(&out_buf[..bytes]) {
                            stdout
                                .write_all(&re.replace_all(
                                    &out_buf[..bytes],
                                    &b"\x1b[35m$0\x1b[m"[..],
                                ))
                                .await
                                .unwrap();
                        } else {
                            stdout.write_all(&out_buf[..bytes]).await.unwrap();
                        }
                        stdout.flush().await.unwrap();
                    }
                    Err(e) => {
                        eprintln!("pty read failed: {:?}", e);
                        break;
                    }
                },
                _ = child.wait() => break,
            }
        }

        Ok(())
    }
}

#[cfg(feature = "async")]
#[tokio::main]
async fn main() {
    use std::os::unix::process::ExitStatusExt as _;

    let mut pty = pty_process::Pty::new().unwrap();
    let pts = pty.pts().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();
    let mut child = pty_process::Command::new("nethack").spawn(&pts).unwrap();
    main::run(&mut child, &mut pty).await.unwrap();
    let status = child.wait().await.unwrap();
    std::process::exit(
        status
            .code()
            .unwrap_or_else(|| status.signal().unwrap_or(0) + 128),
    );
}

#[cfg(not(feature = "async"))]
fn main() {
    unimplemented!()
}
