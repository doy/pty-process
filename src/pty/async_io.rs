use std::os::unix::io::{AsRawFd as _, FromRawFd as _};

pub struct Pty {
    pt: async_io::Async<std::fs::File>,
    ptsname: std::path::PathBuf,
}

impl super::Pty for Pty {
    type Pt = async_io::Async<std::fs::File>;

    fn new() -> crate::error::Result<Self> {
        let (pt_fd, ptsname) = super::create_pt()?;

        // safe because posix_openpt (or the previous functions operating on
        // the result) would have returned an Err (causing us to return early)
        // if the file descriptor was invalid. additionally, into_raw_fd gives
        // up ownership over the file descriptor, allowing the newly created
        // File object to take full ownership.
        let pt = unsafe { std::fs::File::from_raw_fd(pt_fd) };

        let pt = async_io::Async::new(pt)
            .map_err(crate::error::Error::AsyncPty)?;

        Ok(Self { pt, ptsname })
    }

    fn pt(&self) -> &Self::Pt {
        &self.pt
    }

    fn pt_mut(&mut self) -> &mut Self::Pt {
        &mut self.pt
    }

    fn pts(&self) -> crate::error::Result<std::fs::File> {
        let fh = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.ptsname)
            .map_err(|e| {
                crate::error::Error::OpenPts(e, self.ptsname.clone())
            })?;
        Ok(fh)
    }

    fn resize(&self, size: &super::Size) -> crate::error::Result<()> {
        super::set_term_size(self.pt().as_raw_fd(), size)
            .map_err(crate::error::Error::SetTermSize)
    }
}
