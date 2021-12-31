/// Error type for errors from this crate
#[derive(Debug)]
pub enum Error {
    /// error came from std::io::Error
    Io(std::io::Error),
    /// error came from nix::Error
    Nix(nix::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{}", e),
            Self::Nix(e) => write!(f, "{}", e),
        }
    }
}

impl std::convert::From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl std::convert::From<nix::Error> for Error {
    fn from(e: nix::Error) -> Self {
        Self::Nix(e)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Nix(e) => Some(e),
        }
    }
}

/// Convenience wrapper for `Result`s using [`Error`](Error)
pub type Result<T> = std::result::Result<T, Error>;
