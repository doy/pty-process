mod raw_guard;

#[cfg(feature = "async")]
mod main {
    use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
    use tokio_util::compat::FuturesAsyncReadCompatExt as _;

    pub async fn run(
        child: &async_process::Child,
        pty: &pty_process::Pty,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let _raw = super::raw_guard::RawGuard::new();

        let mut in_buf = [0_u8; 4096];
        let mut out_buf = [0_u8; 4096];

        let mut stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut pty = pty.compat();

        loop {
            tokio::select! {
                bytes = stdin.read(&mut in_buf) => match bytes {
                    Ok(bytes) => {
                        pty.write_all(&in_buf[..bytes]).await.unwrap();
                    }
                    Err(e) => {
                        eprintln!("stdin read failed: {:?}", e);
                        break;
                    }
                },
                bytes = pty.read(&mut out_buf) => match bytes {
                    Ok(bytes) => {
                        stdout.write_all(&out_buf[..bytes]).await.unwrap();
                        stdout.flush().await.unwrap();
                    }
                    Err(e) => {
                        eprintln!("pty read failed: {:?}", e);
                        break;
                    }
                },
                _ = child.status_no_drop() => break,
            }
        }

        Ok(())
    }
}

#[cfg(feature = "async")]
#[tokio::main]
async fn main() {
    use std::os::unix::process::ExitStatusExt as _;

    let pty = pty_process::Pty::new().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();
    let mut child = pty_process::Command::new("tac")
        // .args(&["500"])
        .spawn(&pty)
        .unwrap();
    main::run(&child, &pty).await.unwrap();
    let status = child.status().await.unwrap();
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
