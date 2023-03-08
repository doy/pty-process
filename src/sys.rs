use std::os::fd::{AsRawFd as _, FromRawFd as _};

#[derive(Debug)]
pub struct Pty(pub nix::pty::PtyMaster);

impl Pty {
    pub fn open() -> crate::Result<Self> {
        let pt = nix::pty::posix_openpt(
            nix::fcntl::OFlag::O_RDWR
                | nix::fcntl::OFlag::O_NOCTTY
                | nix::fcntl::OFlag::O_CLOEXEC,
        )?;
        nix::pty::grantpt(&pt)?;
        nix::pty::unlockpt(&pt)?;

        Ok(Self(pt))
    }

    pub fn set_term_size(&self, size: crate::Size) -> crate::Result<()> {
        let size = size.into();
        let fd = self.0.as_raw_fd();

        // Safety: nix::pty::PtyMaster is required to contain a valid file
        // descriptor and size is guaranteed to be initialized because it's a
        // normal rust value, and nix::pty::Winsize is a repr(C) struct with
        // the same layout as `struct winsize` from sys/ioctl.h.
        Ok(unsafe {
            set_term_size_unsafe(fd, std::ptr::NonNull::from(&size).as_ptr())
        }
        .map(|_| ())?)
    }

    pub fn pts(&self) -> crate::Result<Pts> {
        Ok(Pts(std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(nix::pty::ptsname_r(&self.0)?)?
            .into()))
    }

    #[cfg(feature = "async")]
    pub fn set_nonblocking(&self) -> nix::Result<()> {
        let bits = nix::fcntl::fcntl(
            self.0.as_raw_fd(),
            nix::fcntl::FcntlArg::F_GETFL,
        )?;
        // Safety: bits was just returned from a F_GETFL call. ideally i would
        // just be able to use from_bits here, but it fails for some reason?
        let mut opts =
            unsafe { nix::fcntl::OFlag::from_bits_unchecked(bits) };
        opts |= nix::fcntl::OFlag::O_NONBLOCK;
        nix::fcntl::fcntl(
            self.0.as_raw_fd(),
            nix::fcntl::FcntlArg::F_SETFL(opts),
        )?;

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
            nix::unistd::setsid()?;
            // Safety: OwnedFds are required to contain a valid file descriptor
            unsafe {
                set_controlling_terminal_unsafe(pts_fd, std::ptr::null())
            }?;
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

nix::ioctl_write_ptr_bad!(
    set_term_size_unsafe,
    libc::TIOCSWINSZ,
    nix::pty::Winsize
);

nix::ioctl_write_ptr_bad!(
    set_controlling_terminal_unsafe,
    libc::TIOCSCTTY,
    libc::c_int
);
