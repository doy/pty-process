use pty_process::Command as _;
use smol::io::{AsyncReadExt as _, AsyncWriteExt as _};
use std::os::unix::io::AsRawFd as _;
use std::os::unix::process::ExitStatusExt as _;

struct RawGuard {
    termios: nix::sys::termios::Termios,
}

impl RawGuard {
    fn new() -> Self {
        let stdin = std::io::stdin().as_raw_fd();
        let termios = nix::sys::termios::tcgetattr(stdin).unwrap();
        let mut termios_raw = termios.clone();
        nix::sys::termios::cfmakeraw(&mut termios_raw);
        nix::sys::termios::tcsetattr(
            stdin,
            nix::sys::termios::SetArg::TCSANOW,
            &termios_raw,
        )
        .unwrap();
        Self { termios }
    }
}

impl Drop for RawGuard {
    fn drop(&mut self) {
        let stdin = std::io::stdin().as_raw_fd();
        let _ = nix::sys::termios::tcsetattr(
            stdin,
            nix::sys::termios::SetArg::TCSANOW,
            &self.termios,
        );
    }
}

async fn run(
    child: &pty_process::smol::Child,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let _raw = RawGuard::new();

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

fn main() {
    let status = smol::block_on(async {
        let mut child = smol::process::Command::new("sleep")
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
