use crate::error::*;

use async_process::unix::CommandExt as _;
use std::os::unix::io::{AsRawFd as _, FromRawFd as _};

impl super::Command<async_process::Child> for async_process::Command {
    fn spawn_pty(
        &mut self,
        size: Option<&crate::pty::Size>,
    ) -> Result<super::Child<async_process::Child>> {
        let (pty, pts, stdin, stdout, stderr) = super::setup_pty(size)?;

        let pt_fd = pty.pt().as_raw_fd();
        let pts_fd = pts.as_raw_fd();

        // safe because the fds are valid (otherwise pty.pts() or dup() would
        // have returned an Err and we would have exited early) and are not
        // owned by any other structure (since dup() returns a fresh copy of
        // the file descriptor), allowing from_raw_fd to take ownership of it.
        self.stdin(unsafe { std::process::Stdio::from_raw_fd(stdin) })
            .stdout(unsafe { std::process::Stdio::from_raw_fd(stdout) })
            .stderr(unsafe { std::process::Stdio::from_raw_fd(stderr) });

        // safe because setsid() and close() are async-signal-safe functions
        // and ioctl() is a raw syscall (which is inherently
        // async-signal-safe).
        unsafe {
            self.pre_exec(move || {
                super::pre_exec(pt_fd, pts_fd, stdin, stdout, stderr)
            });
        }

        let child = self.spawn().map_err(Error::Spawn)?;

        Ok(super::Child { child, pty })
    }
}
