use async_std::io::prelude::WriteExt as _;
use async_std::io::ReadExt as _;
use async_std::prelude::FutureExt as _;
use pty_process::Command as _;
use std::os::unix::process::ExitStatusExt as _;

mod raw_guard;

async fn run(
    child: &pty_process::async_std::Child,
) -> std::result::Result<(), Box<dyn std::error::Error + '_>> {
    let _raw = raw_guard::RawGuard::new();

    let ex = async_executor::Executor::new();

    let input = ex.spawn(async {
        let mut buf = [0_u8; 4096];
        let mut stdin = async_std::io::stdin();
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
        let mut stdout = async_std::io::stdout();
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

    ex.run(input.race(output)).await;

    Ok(())
}

fn main() {
    let status = async_std::task::block_on(async {
        let mut child = async_std::process::Command::new("sleep")
            .args(&["500"])
            .spawn_pty(Some(&pty_process::Size::new(24, 80)))
            .unwrap();
        run(&child).await.unwrap();
        child.status().await.unwrap()
    });
    std::process::exit(
        status
            .code()
            .unwrap_or_else(|| status.signal().unwrap_or(0) + 128),
    );
}
