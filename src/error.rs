#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error creating pty")]
    CreatePtyMaster(#[from] nix::Error),

    #[error("error creating pty")]
    CreatePtySlave(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
