mod raw_guard;

#[cfg(feature = "async")]
mod main {
    use async_std::io::prelude::WriteExt as _;
    use async_std::io::ReadExt as _;
    use async_std::prelude::FutureExt as _;

    pub async fn run(
        child: &async_process::Child,
        pty: &pty_process::Pty,
    ) -> std::result::Result<(), Box<dyn std::error::Error + 'static>> {
        let _raw = super::raw_guard::RawGuard::new();

        let mut input_pty = pty;
        let mut output_pty = pty;
        let ex = async_executor::Executor::new();

        let input = ex.spawn(async {
            let mut buf = [0_u8; 4096];
            let mut stdin = async_std::io::stdin();
            loop {
                match stdin.read(&mut buf).await {
                    Ok(bytes) => {
                        input_pty.write_all(&buf[..bytes]).await.unwrap();
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
            let mut stdout = async_std::io::stdout();
            loop {
                match output_pty.read(&mut buf).await {
                    Ok(bytes) => {
                        stdout.write_all(&buf[..bytes]).await.unwrap();
                        stdout.flush().await.unwrap();
                    }
                    Err(e) => {
                        eprintln!("pty read failed: {:?}", e);
                        break;
                    }
                }
            }
        });

        let wait = async {
            child.status_no_drop().await.unwrap();
        };

        ex.run(input.race(output).race(wait)).await;

        Ok(())
    }
}

#[cfg(feature = "async")]
fn main() {
    use std::os::unix::process::ExitStatusExt as _;

    let status = async_std::task::block_on(async {
        let pty = pty_process::Pty::new().unwrap();
        pty.resize(pty_process::Size::new(24, 80)).unwrap();
        let mut child = pty_process::Command::new("tac")
            // .args(&["500"])
            .spawn(&pty)
            .unwrap();
        main::run(&child, &pty).await.unwrap();
        child.status().await.unwrap()
    });
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
