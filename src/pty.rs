use crate::error::*;

use std::os::unix::io::{AsRawFd as _, FromRawFd as _, IntoRawFd as _};

pub struct Size {
    row: u16,
    col: u16,
    xpixel: u16,
    ypixel: u16,
}

impl Size {
    pub fn new(row: u16, col: u16) -> Self {
        Self {
            row,
            col,
            xpixel: 0,
            ypixel: 0,
        }
    }

    pub fn new_with_pixel(
        row: u16,
        col: u16,
        xpixel: u16,
        ypixel: u16,
    ) -> Self {
        Self {
            row,
            col,
            xpixel,
            ypixel,
        }
    }
}

impl From<&Size> for nix::pty::Winsize {
    fn from(size: &Size) -> Self {
        Self {
            ws_row: size.row,
            ws_col: size.col,
            ws_xpixel: size.xpixel,
            ws_ypixel: size.ypixel,
        }
    }
}

pub struct Pty {
    pt: std::fs::File,
    ptsname: std::path::PathBuf,
}

impl Pty {
    pub fn new() -> Result<Self> {
        let pt = nix::pty::posix_openpt(
            nix::fcntl::OFlag::O_RDWR | nix::fcntl::OFlag::O_NOCTTY,
        )
        .map_err(Error::CreatePty)?;
        nix::pty::grantpt(&pt).map_err(Error::CreatePty)?;
        nix::pty::unlockpt(&pt).map_err(Error::CreatePty)?;

        let ptsname =
            nix::pty::ptsname_r(&pt).map_err(Error::CreatePty)?.into();

        let pt_fd = pt.into_raw_fd();

        // safe because posix_openpt (or the previous functions operating on
        // the result) would have returned an Err (causing us to return early)
        // if the file descriptor was invalid. additionally, into_raw_fd gives
        // up ownership over the file descriptor, allowing the newly created
        // File object to take full ownership.
        let pt = unsafe { std::fs::File::from_raw_fd(pt_fd) };

        Ok(Self { pt, ptsname })
    }

    pub fn pt(&self) -> &std::fs::File {
        &self.pt
    }

    pub fn pts(&self) -> Result<std::fs::File> {
        let fh = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.ptsname)
            .map_err(|e| Error::OpenPts(self.ptsname.clone(), e))?;
        Ok(fh)
    }

    pub fn resize(&self, size: &Size) -> Result<()> {
        let size = size.into();
        let fd = self.pt().as_raw_fd();

        // safe because fd is guaranteed to be valid here (or else the
        // previous open call would have returned an error and exited the
        // function early), and size is guaranteed to be initialized
        // because it's a normal rust value, and nix::pty::Winsize is a
        // repr(C) struct with the same layout as `struct winsize` from
        // sys/ioctl.h.
        unsafe { set_term_size(fd, &size as *const nix::pty::Winsize) }
            .map_err(Error::SetTermSize)?;
        Ok(())
    }
}

nix::ioctl_write_ptr_bad!(set_term_size, libc::TIOCSWINSZ, nix::pty::Winsize);
