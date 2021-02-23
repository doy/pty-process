use crate::error::*;

use std::os::unix::io::FromRawFd as _;

pub struct Pty {
    pt: std::fs::File,
    ptsname: std::path::PathBuf,
}

impl super::Pty for Pty {
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

    fn pt(&self) -> &std::fs::File {
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
}
