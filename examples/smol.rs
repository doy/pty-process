mod raw_guard;

#[cfg(feature = "backend-smol")]
mod main {
    use smol::io::{AsyncReadExt as _, AsyncWriteExt as _};

    pub async fn run(
        child: &pty_process::smol::Child,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let _raw = super::raw_guard::RawGuard::new();

        let ex = smol::Executor::new();

        let input = ex.spawn(async {
            let mut buf = [0_u8; 4096];
            let mut stdin = smol::Unblock::new(std::io::stdin());
            loop {
                match stdin.read(&mut buf).await {
                    Ok(bytes) => {
                        child.pty().write_all(&buf[..bytes]).await.unwrap();
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
            loop {
                match child.pty().read(&mut buf).await {
                    Ok(bytes) => {
                        stdout.write_all(&buf[..bytes]).await.unwrap();
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

        ex.run(smol::future::or(input, output)).await;

        Ok(())
    }
}

#[cfg(feature = "backend-smol")]
fn main() {
    use pty_process::Command as _;
    use std::os::unix::process::ExitStatusExt as _;

    let status = smol::block_on(async {
        let mut child = smol::process::Command::new("sleep")
            .args(&["500"])
            .spawn_pty(Some(&pty_process::Size::new(24, 80)))
            .unwrap();
        main::run(&child).await.unwrap();
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
