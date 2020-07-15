use crate::error::*;

pub struct Pty {
    master: nix::pty::PtyMaster,
    slave: std::fs::File,
}

impl Pty {
    pub fn new() -> Result<Self> {
        let master = nix::pty::posix_openpt(
            nix::fcntl::OFlag::O_RDWR | nix::fcntl::OFlag::O_NOCTTY,
        )?;
        nix::pty::grantpt(&master)?;
        nix::pty::unlockpt(&master)?;

        let name = nix::pty::ptsname_r(&master)?;
        let slave = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(name)?;

        Ok(Self { master, slave })
    }

    pub fn master(&self) -> &nix::pty::PtyMaster {
        &self.master
    }

    pub fn slave(&self) -> &std::fs::File {
        &self.slave
    }
}
