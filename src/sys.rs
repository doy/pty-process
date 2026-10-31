use std::os::{
    fd::{AsRawFd as _, FromRawFd as _},
    unix::prelude::{OpenOptionsExt as _, OsStrExt as _},
};

#[derive(Debug)]
pub struct Pty(std::os::fd::OwnedFd);

impl Pty {
    pub fn open() -> crate::Result<Self> {
        let pt = rustix::pty::openpt(
            // can't use CLOEXEC here because it's linux-specific
            rustix::pty::OpenptFlags::RDWR | rustix::pty::OpenptFlags::NOCTTY,
        )?;
        rustix::pty::grantpt(&pt)?;
        rustix::pty::unlockpt(&pt)?;

        let mut flags = rustix::io::fcntl_getfd(&pt)?;
        flags |= rustix::io::FdFlags::CLOEXEC;
        rustix::io::fcntl_setfd(&pt, flags)?;

        Ok(Self(pt))
    }

    pub unsafe fn from_fd(fd: std::os::fd::OwnedFd) -> Self {
        Self(fd)
    }

    pub fn set_term_size(&self, size: crate::Size) -> crate::Result<()> {
        Ok(rustix::termios::tcsetwinsize(
            &self.0,
            rustix::termios::Winsize::from(size),
        )?)
    }

    pub fn pts(&self) -> crate::Result<Pts> {
        let pts = Pts(std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(
                rustix::fs::OFlags::NOCTTY.bits().try_into().unwrap(),
            )
            .open(std::ffi::OsStr::from_bytes(
                rustix::pty::ptsname(&self.0, vec![])?.as_bytes(),
            ))?
            .into());
        #[cfg(any(target_os = "illumos", target_os = "solaris"))]
        push_streams_modules(&pts.0)?;
        Ok(pts)
    }

    #[cfg(feature = "async")]
    pub fn set_nonblocking(&self) -> rustix::io::Result<()> {
        let mut opts = rustix::fs::fcntl_getfl(&self.0)?;
        opts |= rustix::fs::OFlags::NONBLOCK;
        rustix::fs::fcntl_setfl(&self.0, opts)?;

        Ok(())
    }

    pub fn read_buf<'a>(
        &self,
        buf: &'a mut [std::mem::MaybeUninit<u8>],
    ) -> std::io::Result<(&'a mut [u8], &'a mut [std::mem::MaybeUninit<u8>])>
    {
        rustix::io::read(&self.0, buf).map_err(std::io::Error::from)
    }
}

impl From<Pty> for std::os::fd::OwnedFd {
    fn from(pty: Pty) -> Self {
        let Pty(nix_ptymaster) = pty;
        let raw_fd = nix_ptymaster.as_raw_fd();
        std::mem::forget(nix_ptymaster);

        // Safety: nix::pty::PtyMaster is required to contain a valid file
        // descriptor, and we ensured that the file descriptor will remain
        // valid by skipping the drop implementation for nix::pty::PtyMaster
        unsafe { Self::from_raw_fd(raw_fd) }
    }
}

impl std::os::fd::AsFd for Pty {
    fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
        let raw_fd = self.0.as_raw_fd();

        // Safety: nix::pty::PtyMaster is required to contain a valid file
        // descriptor, and it is owned by self
        unsafe { std::os::fd::BorrowedFd::borrow_raw(raw_fd) }
    }
}

impl std::os::fd::AsRawFd for Pty {
    fn as_raw_fd(&self) -> std::os::fd::RawFd {
        self.0.as_raw_fd()
    }
}

impl std::io::Read for Pty {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        rustix::io::read(&self.0, buf).map_err(std::io::Error::from)
    }
}

impl std::io::Write for Pty {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        rustix::io::write(&self.0, buf).map_err(std::io::Error::from)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl std::io::Read for &Pty {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        rustix::io::read(&self.0, buf).map_err(std::io::Error::from)
    }
}

impl std::io::Write for &Pty {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        rustix::io::write(&self.0, buf).map_err(std::io::Error::from)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// On illumos and Solaris a pty is a pair of STREAMS devices, and the
// subordinate side is a bare stream until the terminal modules are pushed
// onto it: ptem keeps the window size and ldterm is the line discipline.
// Without them TIOCSWINSZ fails with EINVAL and the child gets no termios.
// This mirrors what openpty(3C) does there, and skips the push when autopush
// has already set the stream up.
#[cfg(any(target_os = "illumos", target_os = "solaris"))]
fn push_streams_modules(fd: &std::os::fd::OwnedFd) -> std::io::Result<()> {
    let fd = fd.as_raw_fd();
    // Safety: fd is an open descriptor and each module name is a
    // NUL-terminated string that outlives the call.
    let present = unsafe { libc::ioctl(fd, libc::I_FIND, c"ldterm".as_ptr()) };
    if present < 0 {
        return Err(std::io::Error::last_os_error());
    }
    if present == 0 {
        for module in [c"ptem", c"ldterm", c"ttcompat"] {
            // Safety: as above.
            if unsafe { libc::ioctl(fd, libc::I_PUSH, module.as_ptr()) } < 0 {
                return Err(std::io::Error::last_os_error());
            }
        }
    }
    Ok(())
}

pub struct Pts(std::os::fd::OwnedFd);

impl Pts {
    pub unsafe fn from_fd(fd: std::os::fd::OwnedFd) -> Self {
        Self(fd)
    }

    pub fn setup_subprocess(
        &self,
    ) -> std::io::Result<(
        std::process::Stdio,
        std::process::Stdio,
        std::process::Stdio,
    )> {
        Ok((
            self.0.try_clone()?.into(),
            self.0.try_clone()?.into(),
            self.0.try_clone()?.into(),
        ))
    }

    pub fn session_leader(
        &self,
    ) -> impl FnMut() -> std::io::Result<()> + use<> {
        let pts_fd = self.0.as_raw_fd();
        move || {
            rustix::process::setsid()?;
            rustix::process::ioctl_tiocsctty(unsafe {
                std::os::fd::BorrowedFd::borrow_raw(pts_fd)
            })?;
            Ok(())
        }
    }
}

impl From<Pts> for std::os::fd::OwnedFd {
    fn from(pts: Pts) -> Self {
        pts.0
    }
}

impl std::os::fd::AsFd for Pts {
    fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl std::os::fd::AsRawFd for Pts {
    fn as_raw_fd(&self) -> std::os::fd::RawFd {
        self.0.as_raw_fd()
    }
}
