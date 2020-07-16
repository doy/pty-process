use crate::error::*;

use std::os::unix::io::{AsRawFd as _, FromRawFd as _};
use std::os::unix::process::CommandExt as _;

pub trait Command {
    fn spawn_pty(&mut self) -> Result<Child>;
}

impl Command for std::process::Command {
    fn spawn_pty(&mut self) -> Result<Child> {
        let pty = crate::pty::Pty::new()?;
        let pts = pty.pts()?;

        let pt_fd = pty.pt().as_raw_fd();
        let pts_fd = pts.as_raw_fd();

        self.stdin(unsafe { std::process::Stdio::from_raw_fd(pts_fd) })
            .stdout(unsafe { std::process::Stdio::from_raw_fd(pts_fd) })
            .stderr(unsafe { std::process::Stdio::from_raw_fd(pts_fd) });

        unsafe {
            self.pre_exec(move || {
                // XXX unwrap
                nix::unistd::close(pt_fd)
                    .map_err(|e| e.as_errno().unwrap())?;
                nix::unistd::close(pts_fd)
                    .map_err(|e| e.as_errno().unwrap())?;
                Ok(())
            });
        }

        let child = self.spawn()?;

        Ok(Child { child, pty })
    }
}

pub struct Child {
    child: std::process::Child,
    pty: crate::pty::Pty,
}

impl Child {
    pub fn pty(&self) -> &std::fs::File {
        self.pty.pt()
    }
}

impl std::ops::Deref for Child {
    type Target = std::process::Child;

    fn deref(&self) -> &Self::Target {
        &self.child
    }
}

impl std::ops::DerefMut for Child {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.child
    }
}
