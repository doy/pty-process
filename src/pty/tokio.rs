use crate::error::*;

use std::io::{Read as _, Write as _};
use std::os::unix::io::{AsRawFd as _, FromRawFd as _};

// ideally i would just be able to use tokio::fs::File::from_std on the
// std::fs::File i create from the pty fd, but it appears that tokio::fs::File
// doesn't actually support having both a read and a write operation happening
// on it simultaneously - if you poll the future returned by .read() at any
// point, .write().await will never complete (because it is trying to wait for
// the read to finish before processing the write, which will never happen).
// this unfortunately shows up in patterns like select! pretty frequently, so
// we need to do this the complicated way/:
pub struct Pty {
    pt: tokio::io::unix::AsyncFd<std::fs::File>,
    ptsname: std::path::PathBuf,
}

impl std::ops::Deref for Pty {
    type Target = tokio::io::unix::AsyncFd<std::fs::File>;

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

impl tokio::io::AsyncRead for Pty {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf,
    ) -> std::task::Poll<std::io::Result<()>> {
        loop {
            let mut guard = futures::ready!(self.pt.poll_read_ready(cx))?;
            let mut b = [0u8; 4096];
            match guard.try_io(|inner| inner.get_ref().read(&mut b)) {
                Ok(Ok(bytes)) => {
                    // XXX this is safe, but not particularly efficient
                    buf.clear();
                    buf.initialize_unfilled_to(bytes);
                    buf.set_filled(bytes);
                    buf.filled_mut().copy_from_slice(&b[..bytes]);
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
            let mut guard = futures::ready!(self.pt.poll_write_ready(cx))?;
            match guard.try_io(|inner| inner.get_ref().write(buf)) {
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
            let mut guard = futures::ready!(self.pt.poll_write_ready(cx))?;
            match guard.try_io(|inner| inner.get_ref().flush()) {
                Ok(_) => return std::task::Poll::Ready(Ok(())),
                Err(_would_block) => continue,
            }
        }
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::task::Poll::Ready(Ok(()))
    }
}

impl super::Pty for Pty {
    type Pt = tokio::io::unix::AsyncFd<std::fs::File>;

    fn new() -> Result<Self> {
        let (pt_fd, ptsname) = super::create_pt()?;

        let bits = nix::fcntl::fcntl(pt_fd, nix::fcntl::FcntlArg::F_GETFL)
            .map_err(Error::AsyncPtyNix)?;
        // this should be safe because i am just using the return value of
        // F_GETFL directly, but for whatever reason nix doesn't like
        // from_bits(bits) (it claims it has an unknown field)
        let opts = unsafe {
            nix::fcntl::OFlag::from_bits_unchecked(
                bits | nix::fcntl::OFlag::O_NONBLOCK.bits(),
            )
        };
        nix::fcntl::fcntl(pt_fd, nix::fcntl::FcntlArg::F_SETFL(opts))
            .map_err(Error::AsyncPtyNix)?;

        // safe because posix_openpt (or the previous functions operating on
        // the result) would have returned an Err (causing us to return early)
        // if the file descriptor was invalid. additionally, into_raw_fd gives
        // up ownership over the file descriptor, allowing the newly created
        // File object to take full ownership.
        let pt = unsafe { std::fs::File::from_raw_fd(pt_fd) };

        let pt =
            tokio::io::unix::AsyncFd::new(pt).map_err(Error::AsyncPty)?;

        Ok(Self { pt, ptsname })
    }

    fn pt(&self) -> &Self::Pt {
        &self.pt
    }

    fn pt_mut(&mut self) -> &mut Self::Pt {
        &mut self.pt
    }

    fn pts(&self) -> Result<std::fs::File> {
        let fh = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.ptsname)
            .map_err(|e| Error::OpenPts(e, self.ptsname.clone()))?;
        Ok(fh)
    }

    fn resize(&self, size: &super::Size) -> Result<()> {
        super::set_term_size(self.pt().as_raw_fd(), size)
            .map_err(Error::SetTermSize)
    }
}
