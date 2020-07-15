use crate::error::*;

use std::os::unix::io::{AsRawFd as _, FromRawFd as _};

pub struct Command {
    pty: crate::pty::Pty,
    command: std::process::Command,
}

impl Command {
    pub fn new<S: std::convert::AsRef<std::ffi::OsStr>>(
        program: S,
    ) -> Result<Self> {
        let pty = crate::pty::Pty::new()?;
        let fd = pty.master().as_raw_fd();
        let stdin = unsafe { std::process::Stdio::from_raw_fd(fd) };
        let stdout = unsafe { std::process::Stdio::from_raw_fd(fd) };
        let stderr = unsafe { std::process::Stdio::from_raw_fd(fd) };
        let mut command = std::process::Command::new(program);
        command.stdin(stdin).stdout(stdout).stderr(stderr);
        Ok(Self { pty, command })
    }

    pub fn pty(&self) -> &std::fs::File {
        self.pty.slave()
    }
}

impl std::ops::Deref for Command {
    type Target = std::process::Command;

    fn deref(&self) -> &Self::Target {
        &self.command
    }
}

impl std::ops::DerefMut for Command {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.command
    }
}
