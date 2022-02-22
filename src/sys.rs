use std::os::unix::io::{AsRawFd as _, FromRawFd as _, IntoRawFd as _};

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
            .into_raw_fd()))
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

impl std::os::unix::io::AsRawFd for Pty {
    fn as_raw_fd(&self) -> std::os::unix::io::RawFd {
        self.0.as_raw_fd()
    }
}

pub struct Pts(std::os::unix::io::RawFd);

impl Pts {
    pub fn setup_subprocess(
        &self,
    ) -> nix::Result<(
        std::process::Stdio,
        std::process::Stdio,
        std::process::Stdio,
    )> {
        let pts_fd = self.0.as_raw_fd();

        let stdin = nix::fcntl::fcntl(
            pts_fd,
            nix::fcntl::FcntlArg::F_DUPFD_CLOEXEC(0),
        )?;
        let stdout = nix::fcntl::fcntl(
            pts_fd,
            nix::fcntl::FcntlArg::F_DUPFD_CLOEXEC(0),
        )?;
        let stderr = nix::fcntl::fcntl(
            pts_fd,
            nix::fcntl::FcntlArg::F_DUPFD_CLOEXEC(0),
        )?;

        // Safety: these file descriptors were all just returned from dup, so
        // they must be valid
        Ok((
            unsafe { std::process::Stdio::from_raw_fd(stdin) },
            unsafe { std::process::Stdio::from_raw_fd(stdout) },
            unsafe { std::process::Stdio::from_raw_fd(stderr) },
        ))
    }

    pub fn session_leader(&self) -> impl FnMut() -> std::io::Result<()> {
        let pts_fd = self.0.as_raw_fd();
        move || {
            nix::unistd::setsid()?;
            set_controlling_terminal(pts_fd)?;
            Ok(())
        }
    }
}

impl Drop for Pts {
    fn drop(&mut self) {
        let _ = nix::unistd::close(self.0);
    }
}

impl std::os::unix::io::AsRawFd for Pts {
    fn as_raw_fd(&self) -> std::os::unix::io::RawFd {
        self.0
    }
}

fn set_controlling_terminal(fd: std::os::unix::io::RawFd) -> nix::Result<()> {
    // Safety: Pts is required to contain a valid file descriptor
    unsafe { set_controlling_terminal_unsafe(fd, std::ptr::null()) }
        .map(|_| ())
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
