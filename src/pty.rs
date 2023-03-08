#![allow(clippy::module_name_repetitions)]

use std::io::{Read as _, Write as _};

type AsyncPty = tokio::io::unix::AsyncFd<crate::sys::Pty>;

/// An allocated pty
pub struct Pty(AsyncPty);

impl Pty {
    /// Allocate and return a new pty.
    ///
    /// # Errors
    /// Returns an error if the pty failed to be allocated, or if we were
    /// unable to put it into non-blocking mode.
    pub fn new() -> crate::Result<Self> {
        let pty = crate::sys::Pty::open()?;
        pty.set_nonblocking()?;
        Ok(Self(tokio::io::unix::AsyncFd::new(pty)?))
    }

    /// Change the terminal size associated with the pty.
    ///
    /// # Errors
    /// Returns an error if we were unable to set the terminal size.
    pub fn resize(&self, size: crate::Size) -> crate::Result<()> {
        self.0.get_ref().set_term_size(size)
    }

    /// Opens a file descriptor for the other end of the pty, which should be
    /// attached to the child process running in it. See
    /// [`Command::spawn`](crate::Command::spawn).
    ///
    /// # Errors
    /// Returns an error if the device node to open could not be determined,
    /// or if the device node could not be opened.
    pub fn pts(&self) -> crate::Result<Pts> {
        Ok(Pts(self.0.get_ref().pts()?))
    }

    /// Splits a `Pty` into a read half and a write half, which can be used to
    /// read from and write to the pty concurrently. Does not allocate, but
    /// the returned halves cannot be moved to independent tasks.
    pub fn split(&mut self) -> (ReadPty<'_>, WritePty<'_>) {
        (ReadPty(&self.0), WritePty(&self.0))
    }

    /// Splits a `Pty` into a read half and a write half, which can be used to
    /// read from and write to the pty concurrently. This method requires an
    /// allocation, but the returned halves can be moved to independent tasks.
    /// The original `Pty` instance can be recovered via the
    /// [`OwnedReadPty::unsplit`] method.
    #[must_use]
    pub fn into_split(self) -> (OwnedReadPty, OwnedWritePty) {
        let Self(pt) = self;
        let read_pt = std::sync::Arc::new(pt);
        let write_pt = std::sync::Arc::clone(&read_pt);
        (OwnedReadPty(read_pt), OwnedWritePty(write_pt))
    }
}

impl From<Pty> for std::os::fd::OwnedFd {
    fn from(pty: Pty) -> Self {
        pty.0.into_inner().into()
    }
}

impl std::os::fd::AsFd for Pty {
    fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
        self.0.get_ref().as_fd()
    }
}

impl tokio::io::AsyncRead for Pty {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf,
    ) -> std::task::Poll<std::io::Result<()>> {
        loop {
            let mut guard = match self.0.poll_read_ready(cx) {
                std::task::Poll::Ready(guard) => guard,
                std::task::Poll::Pending => return std::task::Poll::Pending,
            }?;
            // XXX should be able to optimize this once read_buf is stabilized
            // in std
            let b = buf.initialize_unfilled();
            match guard.try_io(|inner| (&inner.get_ref().0).read(b)) {
                Ok(Ok(bytes)) => {
                    buf.advance(bytes);
                    return std::task::Poll::Ready(Ok(()));
                }
                Ok(Err(e)) => return std::task::Poll::Ready(Err(e)),
                Err(_would_block) => continue,
            }
        }
    }
}

impl tokio::io::AsyncWrite for Pty {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        loop {
            let mut guard = match self.0.poll_write_ready(cx) {
                std::task::Poll::Ready(guard) => guard,
                std::task::Poll::Pending => return std::task::Poll::Pending,
            }?;
            match guard.try_io(|inner| (&inner.get_ref().0).write(buf)) {
                Ok(result) => return std::task::Poll::Ready(result),
                Err(_would_block) => continue,
            }
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        loop {
            let mut guard = match self.0.poll_write_ready(cx) {
                std::task::Poll::Ready(guard) => guard,
                std::task::Poll::Pending => return std::task::Poll::Pending,
            }?;
            match guard.try_io(|inner| (&inner.get_ref().0).flush()) {
                Ok(_) => return std::task::Poll::Ready(Ok(())),
                Err(_would_block) => continue,
            }
        }
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
}

/// The child end of the pty
///
/// See [`Pty::pts`] and [`Command::spawn`](crate::Command::spawn)
pub struct Pts(pub(crate) crate::sys::Pts);

/// Borrowed read half of a [`Pty`]
pub struct ReadPty<'a>(&'a AsyncPty);

impl<'a> tokio::io::AsyncRead for ReadPty<'a> {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf,
    ) -> std::task::Poll<std::io::Result<()>> {
        loop {
            let mut guard = match self.0.poll_read_ready(cx) {
                std::task::Poll::Ready(guard) => guard,
                std::task::Poll::Pending => return std::task::Poll::Pending,
            }?;
            // XXX should be able to optimize this once read_buf is stabilized
            // in std
            let b = buf.initialize_unfilled();
            match guard.try_io(|inner| (&inner.get_ref().0).read(b)) {
                Ok(Ok(bytes)) => {
                    buf.advance(bytes);
                    return std::task::Poll::Ready(Ok(()));
                }
                Ok(Err(e)) => return std::task::Poll::Ready(Err(e)),
                Err(_would_block) => continue,
            }
        }
    }
}

/// Borrowed write half of a [`Pty`]
pub struct WritePty<'a>(&'a AsyncPty);

impl<'a> WritePty<'a> {
    /// Change the terminal size associated with the pty.
    ///
    /// # Errors
    /// Returns an error if we were unable to set the terminal size.
    pub fn resize(&self, size: crate::Size) -> crate::Result<()> {
        self.0.get_ref().set_term_size(size)
    }
}

impl<'a> tokio::io::AsyncWrite for WritePty<'a> {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        loop {
            let mut guard = match self.0.poll_write_ready(cx) {
                std::task::Poll::Ready(guard) => guard,
                std::task::Poll::Pending => return std::task::Poll::Pending,
            }?;
            match guard.try_io(|inner| (&inner.get_ref().0).write(buf)) {
                Ok(result) => return std::task::Poll::Ready(result),
                Err(_would_block) => continue,
            }
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        loop {
            let mut guard = match self.0.poll_write_ready(cx) {
                std::task::Poll::Ready(guard) => guard,
                std::task::Poll::Pending => return std::task::Poll::Pending,
            }?;
            match guard.try_io(|inner| (&inner.get_ref().0).flush()) {
                Ok(_) => return std::task::Poll::Ready(Ok(())),
                Err(_would_block) => continue,
            }
        }
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
}

/// Owned read half of a [`Pty`]
#[derive(Debug)]
pub struct OwnedReadPty(std::sync::Arc<AsyncPty>);

impl OwnedReadPty {
    /// Attempt to join the two halves of a `Pty` back into a single instance.
    /// The two halves must have originated from calling
    /// [`into_split`](Pty::into_split) on a single instance.
    ///
    /// # Errors
    /// Returns an error if the two halves came from different [`Pty`]
    /// instances. The mismatched halves are returned as part of the error.
    pub fn unsplit(self, write_half: OwnedWritePty) -> crate::Result<Pty> {
        let Self(read_pt) = self;
        let OwnedWritePty(write_pt) = write_half;
        if std::sync::Arc::ptr_eq(&read_pt, &write_pt) {
            drop(write_pt);
            Ok(Pty(std::sync::Arc::try_unwrap(read_pt)
                // it shouldn't be possible for more than two references to
                // the same pty to exist
                .unwrap_or_else(|_| unreachable!())))
        } else {
            Err(crate::Error::Unsplit(
                Self(read_pt),
                OwnedWritePty(write_pt),
            ))
        }
    }
}

impl tokio::io::AsyncRead for OwnedReadPty {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf,
    ) -> std::task::Poll<std::io::Result<()>> {
        loop {
            let mut guard = match self.0.poll_read_ready(cx) {
                std::task::Poll::Ready(guard) => guard,
                std::task::Poll::Pending => return std::task::Poll::Pending,
            }?;
            // XXX should be able to optimize this once read_buf is stabilized
            // in std
            let b = buf.initialize_unfilled();
            match guard.try_io(|inner| (&inner.get_ref().0).read(b)) {
                Ok(Ok(bytes)) => {
                    buf.advance(bytes);
                    return std::task::Poll::Ready(Ok(()));
                }
                Ok(Err(e)) => return std::task::Poll::Ready(Err(e)),
                Err(_would_block) => continue,
            }
        }
    }
}

/// Owned write half of a [`Pty`]
#[derive(Debug)]
pub struct OwnedWritePty(std::sync::Arc<AsyncPty>);

impl OwnedWritePty {
    /// Change the terminal size associated with the pty.
    ///
    /// # Errors
    /// Returns an error if we were unable to set the terminal size.
    pub fn resize(&self, size: crate::Size) -> crate::Result<()> {
        self.0.get_ref().set_term_size(size)
    }
}

impl tokio::io::AsyncWrite for OwnedWritePty {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        loop {
            let mut guard = match self.0.poll_write_ready(cx) {
                std::task::Poll::Ready(guard) => guard,
                std::task::Poll::Pending => return std::task::Poll::Pending,
            }?;
            match guard.try_io(|inner| (&inner.get_ref().0).write(buf)) {
                Ok(result) => return std::task::Poll::Ready(result),
                Err(_would_block) => continue,
            }
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        loop {
            let mut guard = match self.0.poll_write_ready(cx) {
                std::task::Poll::Ready(guard) => guard,
                std::task::Poll::Pending => return std::task::Poll::Pending,
            }?;
            match guard.try_io(|inner| (&inner.get_ref().0).flush()) {
                Ok(_) => return std::task::Poll::Ready(Ok(())),
                Err(_would_block) => continue,
            }
        }
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
}
