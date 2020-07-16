use crate::error::*;

use std::os::unix::io::{FromRawFd as _, IntoRawFd as _};

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

    pub fn pts(&self) -> Result<std::fs::File> {
        Ok(std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.ptsname)
            .map_err(|e| Error::OpenPts(self.ptsname.clone(), e))?)
    }
}
