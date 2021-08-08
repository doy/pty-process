use pty_process::{Command as _, Pty};
use std::io::{Read as _, Write as _};

#[cfg(feature = "backend-std")]
#[test]
fn test_winch() {
    let (mut child, mut pty) = std::process::Command::new("perl")
        .args(&[
            "-E",
            "$|++; $SIG{WINCH} = sub { say 'WINCH' }; say 'started'; <>",
        ])
        .spawn_pty(Some(&pty_process::Size::new(24, 80)))
        .unwrap();

    let mut buf = [0u8; 1024];
    let bytes = pty.read(&mut buf).unwrap();
    assert_eq!(&buf[..bytes], b"started\r\n");

    pty.resize(&pty_process::Size::new(25, 80)).unwrap();

    let bytes = pty.read(&mut buf).unwrap();
    assert_eq!(&buf[..bytes], b"WINCH\r\n");

    pty.write_all(b"\n").unwrap();
    let status = child.wait().unwrap();
    assert_eq!(status.code().unwrap(), 0);
}
