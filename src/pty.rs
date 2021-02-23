use crate::error::*;

use ::std::os::unix::io::{AsRawFd as _, IntoRawFd as _};

pub mod std;

#[cfg(feature = "async-std")]
pub mod async_io;
#[cfg(feature = "tokio")]
pub mod tokio;

pub trait Pty {
    fn new() -> Result<Self>
    where
        Self: Sized;
    fn pt(&self) -> &::std::fs::File;
    fn pts(&self) -> Result<::std::fs::File>;
    fn resize(&self, size: &super::Size) -> Result<()> {
        set_term_size(self.pt(), size).map_err(Error::SetTermSize)
    }
}

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

fn create_pt() -> Result<(::std::os::unix::io::RawFd, ::std::path::PathBuf)> {
    let pt = nix::pty::posix_openpt(
        nix::fcntl::OFlag::O_RDWR | nix::fcntl::OFlag::O_NOCTTY,
    )
    .map_err(Error::CreatePty)?;
    nix::pty::grantpt(&pt).map_err(Error::CreatePty)?;
    nix::pty::unlockpt(&pt).map_err(Error::CreatePty)?;

    let ptsname = nix::pty::ptsname_r(&pt).map_err(Error::CreatePty)?.into();

    let pt_fd = pt.into_raw_fd();

    Ok((pt_fd, ptsname))
}

nix::ioctl_write_ptr_bad!(
    set_term_size_unsafe,
    libc::TIOCSWINSZ,
    nix::pty::Winsize
);

fn set_term_size(file: &::std::fs::File, size: &Size) -> nix::Result<()> {
    let size = size.into();
    let fd = file.as_raw_fd();
    // safe because std::fs::File is required to contain a valid file
    // descriptor and size is guaranteed to be initialized because it's a
    // normal rust value, and nix::pty::Winsize is a repr(C) struct with the
    // same layout as `struct winsize` from sys/ioctl.h.
    unsafe { set_term_size_unsafe(fd, &size as *const nix::pty::Winsize) }
        .map(|_| ())
}
