use pty_process::Command as _;
use std::os::unix::io::AsRawFd as _;
use std::os::unix::process::ExitStatusExt as _;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

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
    child: &mut pty_process::tokio::Child,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let _raw = RawGuard::new();

    let mut in_buf = [0_u8; 4096];
    let mut out_buf = [0_u8; 4096];

    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();

    loop {
        tokio::select! {
            bytes = stdin.read(&mut in_buf) => match bytes {
                Ok(bytes) => {
                    child.pty_mut().write_all(&in_buf[..bytes]).await.unwrap();
                }
                Err(e) => {
                    eprintln!("stdin read failed: {:?}", e);
                    break;
                }
            },
            bytes = child.pty_mut().read(&mut out_buf) => match bytes {
                Ok(bytes) => {
                    stdout.write_all(&out_buf[..bytes]).await.unwrap();
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
            },
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let mut child = tokio::process::Command::new("sleep")
        .args(&["500"])
        .spawn_pty(Some(&pty_process::Size::new(24, 80)))
        .unwrap();
    run(&mut child).await.unwrap();
    let status = child.wait().await.unwrap();
    std::process::exit(
        status
            .code()
            .unwrap_or_else(|| status.signal().unwrap_or(0) + 128),
    );
}
