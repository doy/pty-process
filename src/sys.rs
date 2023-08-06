use std::os::{
    fd::{AsRawFd as _, FromRawFd as _},
    unix::prelude::OsStrExt as _,
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

    pub fn set_term_size(&self, size: crate::Size) -> crate::Result<()> {
        let size = libc::winsize::from(size);
        let fd = self.0.as_raw_fd();
        // TODO: upstream this to rustix
        let ret = unsafe {
            libc::ioctl(fd, libc::TIOCSWINSZ, std::ptr::addr_of!(size))
        };
        if ret == -1 {
            Err(rustix::io::Errno::from_raw_os_error(
                std::io::Error::last_os_error().raw_os_error().unwrap_or(0),
            )
            .into())
        } else {
            Ok(())
        }
    }

    pub fn pts(&self) -> crate::Result<Pts> {
        Ok(Pts(std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(std::ffi::OsStr::from_bytes(
                rustix::pty::ptsname(&self.0, vec![])?.as_bytes(),
            ))?
            .into()))
    }

    #[cfg(feature = "async")]
    pub fn set_nonblocking(&self) -> rustix::io::Result<()> {
        let mut opts = rustix::fs::fcntl_getfl(&self.0)?;
        opts |= rustix::fs::OFlags::NONBLOCK;
        rustix::fs::fcntl_setfl(&self.0, opts)?;

        Ok(())
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

pub struct Pts(std::os::fd::OwnedFd);

impl Pts {
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

    pub fn session_leader(&self) -> impl FnMut() -> std::io::Result<()> {
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
