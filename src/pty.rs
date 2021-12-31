/// An allocated pty
pub struct Pty {
    pt: async_io::Async<std::fs::File>,
    pts: std::fs::File,
}

impl Pty {
    /// Allocate and return a new pty.
    ///
    /// # Errors
    /// Returns an error if the pty failed to be allocated, or if we were
    /// unable to put it into non-blocking mode.
    pub fn new() -> crate::Result<Self> {
        let (pt, ptsname) = crate::sys::create_pt()?;
        let pt = async_io::Async::new(pt)?;
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
    type Target = async_io::Async<std::fs::File>;

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

// there is an AsyncRead impl for &Async<std::fs::File>, but without this
// explicit impl, rust finds the AsyncRead impl for Async<std::fs::File>
// first, and then complains that it requires &mut self, because method
// resolution/autoderef doesn't take mutability into account
impl futures_io::AsyncRead for &Pty {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        std::pin::Pin::new(&mut &self.pt).poll_read(cx, buf)
    }
}

// same as above
impl futures_io::AsyncWrite for &Pty {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        std::pin::Pin::new(&mut &self.pt).poll_write(cx, buf)
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(&mut &self.pt).poll_flush(cx)
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(&mut &self.pt).poll_close(cx)
    }
}
