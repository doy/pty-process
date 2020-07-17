use crate::error::*;

use std::os::unix::io::{AsRawFd as _, FromRawFd as _};
use std::os::unix::process::CommandExt as _;

pub trait Command {
    fn spawn_pty(&mut self, size: Option<&crate::pty::Size>)
        -> Result<Child>;
}

impl Command for std::process::Command {
    fn spawn_pty(
        &mut self,
        size: Option<&crate::pty::Size>,
    ) -> Result<Child> {
        let pty = crate::pty::Pty::new()?;
        if let Some(size) = size {
            pty.resize(size)?;
        }

        let pts = pty.pts()?;

        let pt_fd = pty.pt().as_raw_fd();
        let pts_fd = pts.as_raw_fd();

        let stdin = nix::unistd::dup(pts_fd).map_err(Error::SpawnNix)?;
        let stdout = nix::unistd::dup(pts_fd).map_err(Error::SpawnNix)?;
        let stderr = nix::unistd::dup(pts_fd).map_err(Error::SpawnNix)?;

        // safe because the fds are valid (otherwise pty.pts() or dup() would
        // have returned an Err and we would have exited early) and are not
        // owned by any other structure (since dup() returns a fresh copy of
        // the file descriptor), allowing from_raw_fd to take ownership of it.
        self.stdin(unsafe { std::process::Stdio::from_raw_fd(stdin) })
            .stdout(unsafe { std::process::Stdio::from_raw_fd(stdout) })
            .stderr(unsafe { std::process::Stdio::from_raw_fd(stderr) });

        // XXX not entirely safe - setsid() and close() are async-signal-safe
        // functions, but ioctl() is not, and only async-signal-safe functions
        // are allowed to be called between fork() and exec(). other things
        // seem to be able to get away with this though, so i'm not sure what
        // the right answer here is?
        unsafe {
            self.pre_exec(move || {
                nix::unistd::setsid().map_err(|e| e.as_errno().unwrap())?;
                set_controlling_terminal(pts_fd, std::ptr::null())
                    .map_err(|e| e.as_errno().unwrap())?;

                // in the parent, destructors will handle closing these file
                // descriptors (other than pt, used by the parent to
                // communicate with the child) when the function ends, but in
                // the child, we end by calling exec(), which doesn't call
                // destructors.

                // XXX unwrap
                nix::unistd::close(pt_fd)
                    .map_err(|e| e.as_errno().unwrap())?;
                nix::unistd::close(pts_fd)
                    .map_err(|e| e.as_errno().unwrap())?;
                // at this point, stdin/stdout/stderr have already been
                // reopened as fds 0/1/2 in the child, so we can (and should)
                // close the originals
                nix::unistd::close(stdin)
                    .map_err(|e| e.as_errno().unwrap())?;
                nix::unistd::close(stdout)
                    .map_err(|e| e.as_errno().unwrap())?;
                nix::unistd::close(stderr)
                    .map_err(|e| e.as_errno().unwrap())?;

                Ok(())
            });
        }

        let child = self.spawn().map_err(Error::Spawn)?;

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

    pub fn pty_resize(&self, size: &crate::pty::Size) -> Result<()> {
        self.pty.resize(size)
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

nix::ioctl_write_ptr_bad!(
    set_controlling_terminal,
    libc::TIOCSCTTY,
    libc::c_int
);
