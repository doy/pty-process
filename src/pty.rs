use crate::error::*;

use std::os::unix::io::{FromRawFd as _, IntoRawFd as _};

pub struct Pty {
    master: std::fs::File,
    slave: std::path::PathBuf,
}

impl Pty {
    pub fn new() -> Result<Self> {
        let master = nix::pty::posix_openpt(
            nix::fcntl::OFlag::O_RDWR | nix::fcntl::OFlag::O_NOCTTY,
        )?;
        nix::pty::grantpt(&master)?;
        nix::pty::unlockpt(&master)?;

        let slave = nix::pty::ptsname_r(&master)?.into();

        let master_fd = master.into_raw_fd();
        let master = unsafe { std::fs::File::from_raw_fd(master_fd) };

        Ok(Self { master, slave })
    }

    pub fn master(&self) -> &std::fs::File {
        &self.master
    }

    pub fn slave(&self) -> Result<std::fs::File> {
        Ok(std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.slave)?)
    }
}
