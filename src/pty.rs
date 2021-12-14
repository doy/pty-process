use ::std::os::unix::io::IntoRawFd as _;

#[cfg(any(feature = "backend-async-std", feature = "backend-smol"))]
pub mod async_io;
#[cfg(feature = "backend-std")]
pub mod std;
#[cfg(feature = "backend-tokio")]
pub mod tokio;

pub trait Pty {
    type Pt;

    fn new() -> crate::error::Result<Self>
    where
        Self: Sized;
    fn pt(&self) -> &Self::Pt;
    fn pt_mut(&mut self) -> &mut Self::Pt;
    fn pts(&self) -> crate::error::Result<::std::fs::File>;
    fn resize(&self, size: &super::Size) -> crate::error::Result<()>;
}

/// Represents the size of the pty.
pub struct Size {
    row: u16,
    col: u16,
    xpixel: u16,
    ypixel: u16,
}

impl Size {
    /// Returns a [`Size`](Size) instance with the given number of rows and
    /// columns.
    #[must_use]
    pub fn new(row: u16, col: u16) -> Self {
        Self {
            row,
            col,
            xpixel: 0,
            ypixel: 0,
        }
    }

    /// Returns a [`Size`](Size) instance with the given number of rows and
    /// columns, as well as the given pixel dimensions.
    #[must_use]
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

fn create_pt(
) -> crate::error::Result<(::std::os::unix::io::RawFd, ::std::path::PathBuf)>
{
    let pt = nix::pty::posix_openpt(
        nix::fcntl::OFlag::O_RDWR | nix::fcntl::OFlag::O_NOCTTY,
    )
    .map_err(crate::error::create_pty)?;
    nix::pty::grantpt(&pt).map_err(crate::error::create_pty)?;
    nix::pty::unlockpt(&pt).map_err(crate::error::create_pty)?;

    let ptsname = nix::pty::ptsname_r(&pt)
        .map_err(crate::error::create_pty)?
        .into();

    let pt_fd = pt.into_raw_fd();

    Ok((pt_fd, ptsname))
}

nix::ioctl_write_ptr_bad!(
    set_term_size_unsafe,
    libc::TIOCSWINSZ,
    nix::pty::Winsize
);

fn set_term_size(
    fd: ::std::os::unix::io::RawFd,
    size: &Size,
) -> nix::Result<()> {
    let size = size.into();
    // safe because std::fs::File is required to contain a valid file
    // descriptor and size is guaranteed to be initialized because it's a
    // normal rust value, and nix::pty::Winsize is a repr(C) struct with the
    // same layout as `struct winsize` from sys/ioctl.h.
    unsafe {
        set_term_size_unsafe(fd, ::std::ptr::NonNull::from(&size).as_ptr())
    }
    .map(|_| ())
}
