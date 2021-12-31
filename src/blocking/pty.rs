/// An allocated pty
pub struct Pty {
    pt: std::fs::File,
    pts: std::fs::File,
}

impl Pty {
    /// Allocate and return a new pty.
    ///
    /// # Errors
    /// Returns an error if the pty failed to be allocated.
    pub fn new() -> crate::Result<Self> {
        let (pt, ptsname) = crate::sys::create_pt()?;
        let pts = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&ptsname)?;
        Ok(Self { pt, pts })
    }

    /// Change the terminal size associated with the pty.
    ///
    /// # Errors
    /// Returns an error if we were unable to set the terminal size.
    pub fn resize(&self, size: crate::Size) -> crate::Result<()> {
        Ok(crate::sys::set_term_size(self, size)?)
    }

    pub(crate) fn pts(&self) -> &std::fs::File {
        &self.pts
    }
}

impl std::ops::Deref for Pty {
    type Target = std::fs::File;

    fn deref(&self) -> &Self::Target {
        &self.pt
    }
}

impl std::ops::DerefMut for Pty {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.pt
    }
}

impl std::os::unix::io::AsRawFd for Pty {
    fn as_raw_fd(&self) -> std::os::unix::io::RawFd {
        self.pt.as_raw_fd()
    }
}

// there is a Read impl for &std::fs::File, but without this explicit impl,
// rust finds the Read impl for std::fs::File first, and then complains that
// it requires &mut self, because method resolution/autoderef doesn't take
// mutability into account
impl std::io::Read for &Pty {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        (&self.pt).read(buf)
    }
}

// same as above
impl std::io::Write for &Pty {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        (&self.pt).write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        (&self.pt).flush()
    }
}
