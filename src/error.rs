/// Error type for this crate
#[derive(Debug)]
pub enum Error {
    /// error making pty async
    AsyncPty(std::io::Error),

    /// error making pty async
    AsyncPtyNix(nix::Error),

    /// error creating pty
    CreatePty(nix::Error),

    /// error opening pts at \<path\>
    OpenPts(std::io::Error, std::path::PathBuf),

    /// error setting terminal size
    SetTermSize(nix::Error),

    /// error spawning subprocess
    Spawn(std::io::Error),

    /// error spawning subprocess
    SpawnNix(nix::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AsyncPty(e) => {
                write!(f, "error making pty async: {}", e)
            }
            Self::AsyncPtyNix(e) => {
                write!(f, "error making pty async: {}", e)
            }
            Self::CreatePty(e) => {
                write!(f, "error creating pty: {}", e)
            }
            Self::OpenPts(e, path) => {
                write!(f, "error opening pts at {}: {}", path.display(), e)
            }
            Self::SetTermSize(e) => {
                write!(f, "error setting terminal size: {}", e)
            }
            Self::Spawn(e) => {
                write!(f, "error spawning subprocess: {}", e)
            }
            Self::SpawnNix(e) => {
                write!(f, "error spawning subprocess: {}", e)
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::AsyncPty(e) | Self::Spawn(e) | Self::OpenPts(e, _) => {
                Some(e)
            }
            Self::AsyncPtyNix(e) | Self::SpawnNix(e) | Self::CreatePty(e) => {
                Some(e)
            }
            Self::SetTermSize(e) => Some(e),
        }
    }
}

/// Convenience wrapper for `Result`s using [`Error`](Error)
pub type Result<T> = std::result::Result<T, Error>;
