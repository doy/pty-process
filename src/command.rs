use crate::error::*;

use std::os::unix::io::{AsRawFd as _, FromRawFd as _};
use std::os::unix::process::CommandExt as _;

pub struct Command {
    pty: crate::pty::Pty,
    command: std::process::Command,
    slave_fh: Option<std::fs::File>,
}

impl Command {
    pub fn new<S: AsRef<std::ffi::OsStr>>(program: S) -> Result<Self> {
        let pty = crate::pty::Pty::new()?;
        let master_fd = pty.master().as_raw_fd();
        let slave_fh = pty.slave()?;
        let slave_fd = slave_fh.as_raw_fd();
        let mut command = std::process::Command::new(program);
        command
            .stdin(unsafe { std::process::Stdio::from_raw_fd(slave_fd) })
            .stdout(unsafe { std::process::Stdio::from_raw_fd(slave_fd) })
            .stderr(unsafe { std::process::Stdio::from_raw_fd(slave_fd) });
        unsafe {
            command.pre_exec(move || {
                // XXX unwrap
                nix::unistd::close(master_fd)
                    .map_err(|e| e.as_errno().unwrap())?;
                nix::unistd::close(slave_fd)
                    .map_err(|e| e.as_errno().unwrap())?;
                Ok(())
            });
        }
        Ok(Self {
            pty,
            command,
            slave_fh: Some(slave_fh),
        })
    }

    pub fn pty(&self) -> &std::fs::File {
        self.pty.master()
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        self.command.args(args);
        self
    }

    pub fn spawn(&mut self) -> Result<std::process::Child> {
        let child = self.command.spawn()?;
        self.slave_fh = None;
        Ok(child)
    }
}
