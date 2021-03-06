/// Error type for this crate
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// error making pty async
    #[error("error making pty async")]
    AsyncPty(#[source] std::io::Error),

    /// error making pty async
    #[error("error making pty async")]
    AsyncPtyNix(#[source] nix::Error),

    /// error creating pty
    #[error("error creating pty")]
    CreatePty(#[source] nix::Error),

    /// error opening pts at \<path\>
    #[error("error opening pts at {0}")]
    OpenPts(#[source] std::io::Error, std::path::PathBuf),

    /// error setting terminal size
    #[error("error setting terminal size")]
    SetTermSize(#[source] nix::Error),

    /// error spawning subprocess
    #[error("error spawning subprocess")]
    Spawn(#[source] std::io::Error),

    /// error spawning subprocess
    #[error("error spawning subprocess")]
    SpawnNix(#[source] nix::Error),
}

/// Convenience wrapper for `Result`s using [`Error`](Error)
pub type Result<T> = std::result::Result<T, Error>;
