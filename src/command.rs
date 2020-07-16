use crate::error::*;

use std::os::unix::io::{AsRawFd as _, FromRawFd as _};
use std::os::unix::process::CommandExt as _;

pub struct Command {
    pty: crate::pty::Pty,
    command: std::process::Command,
    pts_fh: Option<std::fs::File>,
}

impl Command {
    pub fn new<S: AsRef<std::ffi::OsStr>>(program: S) -> Result<Self> {
        let pty = crate::pty::Pty::new()?;
        let pt_fd = pty.pt().as_raw_fd();
        let pts_fh = pty.pts()?;
        let pts_fd = pts_fh.as_raw_fd();
        let mut command = std::process::Command::new(program);
        command
            .stdin(unsafe { std::process::Stdio::from_raw_fd(pts_fd) })
            .stdout(unsafe { std::process::Stdio::from_raw_fd(pts_fd) })
            .stderr(unsafe { std::process::Stdio::from_raw_fd(pts_fd) });
        unsafe {
            command.pre_exec(move || {
                // XXX unwrap
                nix::unistd::close(pt_fd)
                    .map_err(|e| e.as_errno().unwrap())?;
                nix::unistd::close(pts_fd)
                    .map_err(|e| e.as_errno().unwrap())?;
                Ok(())
            });
        }
        Ok(Self {
            pty,
            command,
            pts_fh: Some(pts_fh),
        })
    }

    pub fn pty(&self) -> &std::fs::File {
        self.pty.pt()
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
        self.pts_fh = None;
        Ok(child)
    }
}
