use std::io::{Read as _, Write as _};

fn main() {
    let mut cmd = pty_process::Command::new("ls").unwrap();
    cmd.args(&["--color=auto"]);
    let mut child = cmd.spawn().unwrap();
    let mut buf = [0_u8; 1];
    loop {
        cmd.pty().read_exact(&mut buf).unwrap();
        print!("{}", buf[0] as char);
        std::io::stdout().flush().unwrap();
    }
    child.wait().unwrap();
}
