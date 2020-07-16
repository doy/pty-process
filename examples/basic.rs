use std::io::{Read as _, Write as _};
use std::os::unix::io::AsRawFd as _;

use pty_process::Command as _;

fn main() {
    let mut child = std::process::Command::new("cat")
        // .args(&["--color=auto"])
        .spawn_pty()
        .unwrap();
    let mut buf = [0_u8; 4096];
    let pty = child.pty().as_raw_fd();
    let stdin = std::io::stdin().as_raw_fd();
    loop {
        let mut set = nix::sys::select::FdSet::new();
        set.insert(pty);
        set.insert(stdin);
        match nix::sys::select::select(None, Some(&mut set), None, None, None)
        {
            Ok(n) => {
                if n > 0 {
                    if set.contains(pty) {
                        match child.pty().read(&mut buf) {
                            Ok(bytes) => {
                                let buf = &buf[..bytes];
                                print!(
                                    "got {} bytes: '{}'",
                                    bytes,
                                    std::str::from_utf8(buf).unwrap()
                                );
                                std::io::stdout().flush().unwrap();
                            }
                            Err(e) => {
                                eprintln!("pty read failed: {:?}", e);
                                break;
                            }
                        };
                    }
                    if set.contains(stdin) {
                        match std::io::stdin().read(&mut buf) {
                            Ok(bytes) => {
                                let buf = &buf[..bytes];
                                child.pty().write_all(buf).unwrap();
                            }
                            Err(e) => {
                                eprintln!("stdin read failed: {:?}", e);
                                break;
                            }
                        }
                    }
                }
            }
            Err(e) => println!("select failed: {:?}", e),
        }
    }
    child.wait().unwrap();
}
