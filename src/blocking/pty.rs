/// Allocate and return a new pty.
///
/// # Errors
/// Returns an error if the pty failed to be allocated.
pub fn open() -> crate::Result<(Pty, Pts)> {
    let pty = crate::sys::Pty::open()?;
    let pts = pty.pts()?;
    Ok((Pty(pty), Pts(pts)))
}

/// An allocated pty
pub struct Pty(crate::sys::Pty);

impl Pty {
    /// Change the terminal size associated with the pty.
    ///
    /// # Errors
    /// Returns an error if we were unable to set the terminal size.
    pub fn resize(&self, size: crate::Size) -> crate::Result<()> {
        self.0.set_term_size(size)
    }
}

impl From<Pty> for std::os::fd::OwnedFd {
    fn from(pty: Pty) -> Self {
        pty.0.into()
    }
}

impl std::os::fd::AsFd for Pty {
    fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl std::os::fd::AsRawFd for Pty {
    fn as_raw_fd(&self) -> std::os::fd::RawFd {
        self.0.as_raw_fd()
    }
}

impl std::io::Read for Pty {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }
}

impl std::io::Write for Pty {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

impl std::io::Read for &Pty {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        (&self.0).read(buf)
    }
}

impl std::io::Write for &Pty {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        (&self.0).write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        (&self.0).flush()
    }
}

/// The child end of the pty
///
/// See [`Pty::pts`] and [`Command::spawn`](crate::blocking::Command::spawn)
pub struct Pts(pub(crate) crate::sys::Pts);
