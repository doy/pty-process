use ::std::os::unix::io::{AsRawFd as _, FromRawFd as _};

#[cfg(any(feature = "backend-async-std", feature = "backend-smol"))]
mod async_process;
#[cfg(feature = "backend-std")]
mod std;
#[cfg(feature = "backend-tokio")]
mod tokio;

pub struct Command<C: Impl, P: crate::pty::Impl> {
    inner: C,
    stdin_set: bool,
    stdout_set: bool,
    stderr_set: bool,

    _phantom: ::std::marker::PhantomData<P>,
}

impl<C: Impl, P: crate::pty::Impl> Command<C, P> {
    pub fn new<S: AsRef<::std::ffi::OsStr>>(program: S) -> Self {
        Self {
            inner: C::new_impl(program.as_ref()),
            stdin_set: false,
            stdout_set: false,
            stderr_set: false,

            _phantom: ::std::marker::PhantomData,
        }
    }

    pub fn stdin<T: Into<::std::process::Stdio>>(&mut self, cfg: T) {
        self.stdin_set = true;
        self.inner.stdin_impl(cfg.into());
    }

    pub fn stdout<T: Into<::std::process::Stdio>>(&mut self, cfg: T) {
        self.stdout_set = true;
        self.inner.stdout_impl(cfg.into());
    }

    pub fn stderr<T: Into<::std::process::Stdio>>(&mut self, cfg: T) {
        self.stderr_set = true;
        self.inner.stderr_impl(cfg.into());
    }

    pub fn spawn(
        &mut self,
        pty: crate::Pty<P>,
    ) -> crate::Result<Child<<C as Impl>::Child, P>> {
        let (pts, stdin, stdout, stderr) = pty.setup()?;

        let pt_fd = pty.pt().as_raw_fd();
        let pts_fd = pts.as_raw_fd();

        self.inner
            .stdin_impl(unsafe { ::std::process::Stdio::from_raw_fd(stdin) });
        self.inner.stdout_impl(unsafe {
            ::std::process::Stdio::from_raw_fd(stdout)
        });
        self.inner.stderr_impl(unsafe {
            ::std::process::Stdio::from_raw_fd(stderr)
        });

        let pre_exec = move || {
            nix::unistd::setsid()?;
            set_controlling_terminal(pts_fd)?;

            // in the parent, destructors will handle closing these file
            // descriptors (other than pt, used by the parent to
            // communicate with the child) when the function ends, but in
            // the child, we end by calling exec(), which doesn't call
            // destructors.

            nix::unistd::close(pt_fd)?;
            nix::unistd::close(pts_fd)?;
            // at this point, stdin/stdout/stderr have already been
            // reopened as fds 0/1/2 in the child, so we can (and should)
            // close the originals
            nix::unistd::close(stdin)?;
            nix::unistd::close(stdout)?;
            nix::unistd::close(stderr)?;

            Ok(())
        };
        // safe because setsid() and close() are async-signal-safe functions
        // and ioctl() is a raw syscall (which is inherently
        // async-signal-safe).
        unsafe { self.inner.pre_exec_impl(pre_exec) };

        let child = self.inner.spawn_impl()?;

        Ok(Child::new(child, pty))
    }
}

impl<C: Impl, P: crate::pty::Impl> ::std::ops::Deref for Command<C, P> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<C: Impl, P: crate::pty::Impl> ::std::ops::DerefMut for Command<C, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct Child<C, P: crate::pty::Impl> {
    inner: C,
    pty: crate::Pty<P>,
}

impl<C, P: crate::pty::Impl> Child<C, P> {
    fn new(inner: C, pty: crate::Pty<P>) -> Self {
        Self { inner, pty }
    }

    /// Returns a reference to the pty.
    ///
    /// The underlying pty instance is guaranteed to implement
    /// [`AsRawFd`](::std::os::unix::io::AsRawFd), as well as the appropriate
    /// `Read` and `Write` traits for the associated backend.
    pub fn pty(&self) -> &P {
        self.pty.pt()
    }

    /// Returns a mutable reference to the pty.
    ///
    /// The underlying pty instance is guaranteed to implement
    /// [`AsRawFd`](::std::os::unix::io::AsRawFd), as well as the appropriate
    /// `Read` and `Write` traits for the associated backend.
    ///
    /// This method is primarily useful for the tokio backend, since tokio's
    /// `AsyncRead` and `AsyncWrite` traits have methods which take mutable
    /// references.
    pub fn pty_mut(&mut self) -> &mut P {
        self.pty.pt_mut()
    }

    /// Causes the pty to change its size.
    ///
    /// This will additionally cause a `SIGWINCH` signal to be sent to the
    /// running process.
    ///
    /// # Errors
    /// * `Error::SetTermSize`: error setting terminal size
    pub fn resize_pty(
        &self,
        size: crate::pty::Size,
    ) -> crate::error::Result<()> {
        self.pty.resize(size)
    }
}

impl<C, P: crate::pty::Impl> ::std::ops::Deref for Child<C, P> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<C, P: crate::pty::Impl> ::std::ops::DerefMut for Child<C, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub trait Impl {
    type Child;

    fn new_impl(program: &::std::ffi::OsStr) -> Self;
    fn stdin_impl(&mut self, cfg: ::std::process::Stdio);
    fn stdout_impl(&mut self, cfg: ::std::process::Stdio);
    fn stderr_impl(&mut self, cfg: ::std::process::Stdio);
    unsafe fn pre_exec_impl<F>(&mut self, f: F)
    where
        F: FnMut() -> ::std::io::Result<()> + Send + Sync + 'static;

    fn spawn_impl(&mut self) -> crate::Result<Self::Child>;
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
