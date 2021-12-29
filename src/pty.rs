pub struct Pty {
    pt: async_io::Async<std::fs::File>,
    ptsname: std::path::PathBuf,
}

impl Pty {
    pub fn new() -> crate::Result<Self> {
        let (pt, ptsname) =
            crate::sys::create_pt().map_err(crate::error::create_pty)?;
        let pt =
            async_io::Async::new(pt).map_err(crate::error::create_pty)?;
        Ok(Self { pt, ptsname })
    }

    pub fn resize(&self, size: crate::Size) -> crate::error::Result<()> {
        crate::sys::set_term_size(self, size)
            .map_err(crate::error::set_term_size)
    }

    pub(crate) fn pts(&self) -> std::io::Result<std::fs::File> {
        let fh = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.ptsname)?;
        Ok(fh)
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
