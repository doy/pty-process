use crate::error::*;

use std::os::unix::io::{AsRawFd as _, FromRawFd as _};

pub struct Pty {
    pt: std::fs::File,
    ptsname: std::path::PathBuf,
}

impl super::Pty for Pty {
    type Pt = std::fs::File;

    fn new() -> Result<Self> {
        let (pt_fd, ptsname) = super::create_pt()?;

        // safe because posix_openpt (or the previous functions operating on
        // the result) would have returned an Err (causing us to return early)
        // if the file descriptor was invalid. additionally, into_raw_fd gives
        // up ownership over the file descriptor, allowing the newly created
        // File object to take full ownership.
        let pt = unsafe { std::fs::File::from_raw_fd(pt_fd) };

        Ok(Self { pt, ptsname })
    }

    fn pt(&self) -> &Self::Pt {
        &self.pt
    }

    fn pts(&self) -> Result<std::fs::File> {
        let fh = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.ptsname)
            .map_err(|e| Error::OpenPts(self.ptsname.clone(), e))?;
        Ok(fh)
    }

    fn resize(&self, size: &super::Size) -> Result<()> {
        super::set_term_size(self.pt().as_raw_fd(), size)
            .map_err(Error::SetTermSize)
    }
}
