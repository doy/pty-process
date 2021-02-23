use crate::error::*;

use ::std::os::unix::io::AsRawFd as _;

mod std;

#[cfg(feature = "async-process")]
mod async_process;
#[cfg(feature = "tokio")]
mod tokio;

pub trait Command {
    type Child;

    fn spawn_pty(
        &mut self,
        size: Option<&crate::pty::Size>,
    ) -> Result<Child<Self::Child>>;
}

impl<T> Command for T
where
    T: CommandImpl,
{
    type Child = T::Child;

    fn spawn_pty(
        &mut self,
        size: Option<&crate::pty::Size>,
    ) -> Result<Child<Self::Child>> {
        let (pty, pts, stdin, stdout, stderr) = setup_pty(size)?;

        let pt_fd = pty.pt().as_raw_fd();
        let pts_fd = pts.as_raw_fd();

        self.std_fds(stdin, stdout, stderr);

        let pre_exec = move || {
            nix::unistd::setsid().map_err(|e| e.as_errno().unwrap())?;
            set_controlling_terminal(pts_fd)
                .map_err(|e| e.as_errno().unwrap())?;

            // in the parent, destructors will handle closing these file
            // descriptors (other than pt, used by the parent to
            // communicate with the child) when the function ends, but in
            // the child, we end by calling exec(), which doesn't call
            // destructors.

            // XXX unwrap
            nix::unistd::close(pt_fd).map_err(|e| e.as_errno().unwrap())?;
            nix::unistd::close(pts_fd).map_err(|e| e.as_errno().unwrap())?;
            // at this point, stdin/stdout/stderr have already been
            // reopened as fds 0/1/2 in the child, so we can (and should)
            // close the originals
            nix::unistd::close(stdin).map_err(|e| e.as_errno().unwrap())?;
            nix::unistd::close(stdout).map_err(|e| e.as_errno().unwrap())?;
            nix::unistd::close(stderr).map_err(|e| e.as_errno().unwrap())?;

            Ok(())
        };
        // safe because setsid() and close() are async-signal-safe functions
        // and ioctl() is a raw syscall (which is inherently
        // async-signal-safe).
        unsafe { self.pre_exec_impl(pre_exec) };

        let child = self.spawn_impl().map_err(Error::Spawn)?;

        Ok(Child { child, pty })
    }
}

pub struct Child<T> {
    child: T,
    pty: crate::pty::Pty,
}

impl<T> Child<T> {
    pub fn pty(&self) -> &::std::fs::File {
        self.pty.pt()
    }

    pub fn pty_resize(&self, size: &crate::pty::Size) -> Result<()> {
        self.pty.resize(size)
    }
}

impl<T> ::std::ops::Deref for Child<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.child
    }
}

impl<T> ::std::ops::DerefMut for Child<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.child
    }
}

// XXX shouldn't be pub?
pub trait CommandImpl {
    type Child;

    fn std_fds(
        &mut self,
        stdin: ::std::os::unix::io::RawFd,
        stdout: ::std::os::unix::io::RawFd,
        stderr: ::std::os::unix::io::RawFd,
    );
    unsafe fn pre_exec_impl<F>(&mut self, f: F)
    where
        F: FnMut() -> ::std::io::Result<()> + Send + Sync + 'static;
    fn spawn_impl(&mut self) -> ::std::io::Result<Self::Child>;
}

fn setup_pty(
    size: Option<&crate::pty::Size>,
) -> Result<(
    crate::pty::Pty,
    ::std::fs::File,
    ::std::os::unix::io::RawFd,
    ::std::os::unix::io::RawFd,
    ::std::os::unix::io::RawFd,
)> {
    let pty = crate::pty::Pty::new()?;
    if let Some(size) = size {
        pty.resize(size)?;
    }

    let pts = pty.pts()?;
    let pts_fd = pts.as_raw_fd();

    let stdin = nix::unistd::dup(pts_fd).map_err(Error::SpawnNix)?;
    let stdout = nix::unistd::dup(pts_fd).map_err(Error::SpawnNix)?;
    let stderr = nix::unistd::dup(pts_fd).map_err(Error::SpawnNix)?;

    Ok((pty, pts, stdin, stdout, stderr))
}

fn set_controlling_terminal(
    fd: ::std::os::unix::io::RawFd,
) -> nix::Result<()> {
    // safe because std::fs::File is required to contain a valid file
    // descriptor
    unsafe { set_controlling_terminal_unsafe(fd, ::std::ptr::null()) }
        .map(|_| ())
}

nix::ioctl_write_ptr_bad!(
    set_controlling_terminal_unsafe,
    libc::TIOCSCTTY,
    libc::c_int
);
