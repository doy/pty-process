use ::std::os::unix::io::{AsRawFd as _, FromRawFd as _, IntoRawFd as _};

#[cfg(any(feature = "backend-async-std", feature = "backend-smol"))]
pub mod async_io;
#[cfg(feature = "backend-std")]
pub mod std;
#[cfg(feature = "backend-tokio")]
pub mod tokio;

pub struct Pty<Pt: Impl> {
    pt: Pt,
    ptsname: ::std::path::PathBuf,
}

impl<Pt: Impl> Pty<Pt> {
    pub fn new() -> crate::error::Result<Self> {
        let (pt_fd, ptsname) = create_pt()?;

        // safe because posix_openpt (or the previous functions operating on
        // the result) would have returned an Err (causing us to return early)
        // if the file descriptor was invalid. additionally, into_raw_fd gives
        // up ownership over the file descriptor, allowing the newly created
        // File object to take full ownership.
        let fh = unsafe { ::std::fs::File::from_raw_fd(pt_fd) };
        let pt = Pt::new_from_fh(fh)?;

        Ok(Self { pt, ptsname })
    }

    pub fn pt(&self) -> &Pt {
        &self.pt
    }

    pub fn pt_mut(&mut self) -> &mut Pt {
        &mut self.pt
    }

    pub fn resize(&self, size: crate::Size) -> crate::error::Result<()> {
        set_term_size(self.pt().as_raw_fd(), size)
            .map_err(crate::error::set_term_size)
    }

    fn pts(&self) -> crate::error::Result<::std::fs::File> {
        let fh = ::std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.ptsname)
            .map_err(crate::error::create_pty)?;
        Ok(fh)
    }

    pub(crate) fn setup(
        &self,
    ) -> crate::Result<(
        ::std::fs::File,
        ::std::os::unix::io::RawFd,
        ::std::os::unix::io::RawFd,
        ::std::os::unix::io::RawFd,
    )> {
        let pts = self.pts()?;
        let pts_fd = pts.as_raw_fd();

        let stdin =
            nix::unistd::dup(pts_fd).map_err(crate::error::create_pty)?;
        let stdout =
            nix::unistd::dup(pts_fd).map_err(crate::error::create_pty)?;
        let stderr =
            nix::unistd::dup(pts_fd).map_err(crate::error::create_pty)?;

        Ok((pts, stdin, stdout, stderr))
    }
}

pub trait Impl: ::std::os::unix::io::AsRawFd + Sized {
    fn new_from_fh(fh: ::std::fs::File) -> crate::Result<Self>;
}

/// Represents the size of the pty.
#[derive(Debug, Clone, Copy)]
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
    size: Size,
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
