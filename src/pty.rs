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

impl From<Size> for nix::pty::Winsize {
    fn from(size: Size) -> Self {
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
        let pt = unsafe { std::fs::File::from_raw_fd(pt_fd) };

        Ok(Self { pt, ptsname })
    }

    pub fn pt(&self) -> &std::fs::File {
        &self.pt
    }

    pub fn pts(&self, size: Option<Size>) -> Result<std::fs::File> {
        let fh = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.ptsname)
            .map_err(|e| Error::OpenPts(self.ptsname.clone(), e))?;
        let fd = fh.as_raw_fd();
        if let Some(size) = size {
            let size = size.into();
            unsafe {
                set_term_size(fd, &size as *const nix::pty::Winsize)
                    .map_err(Error::SetTermSize)?;
            }
        }
        Ok(fh)
    }
}

nix::ioctl_write_ptr_bad!(set_term_size, libc::TIOCSWINSZ, nix::pty::Winsize);
