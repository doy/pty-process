#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error creating pty")]
    CreatePty(#[source] nix::Error),

    #[error("error opening pts at {0}")]
    OpenPts(std::path::PathBuf, #[source] std::io::Error),

    #[error("error setting terminal size")]
    SetTermSize(#[source] nix::Error),

    #[error("error spawning subprocess")]
    Spawn(#[source] std::io::Error),

    #[error("error spawning subprocess")]
    SpawnNix(#[source] nix::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
