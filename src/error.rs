/// Source for errors in this crate
#[derive(Debug)]
pub enum Source {
    /// error came from std::io::Error
    Io(std::io::Error),
    /// error came from nix::Error
    Nix(nix::Error),
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{}", e),
            Self::Nix(e) => write!(f, "{}", e),
        }
    }
}

impl std::convert::From<std::io::Error> for Source {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl std::convert::From<nix::Error> for Source {
    fn from(e: nix::Error) -> Self {
        Self::Nix(e)
    }
}

/// Error type for this crate
#[derive(Debug)]
pub enum Error {
    /// error creating pty
    CreatePty(Source),

    /// error setting terminal size
    SetTermSize(Source),

    /// error spawning subprocess
    Spawn(Source),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CreatePty(e) => {
                write!(f, "error creating pty: {}", e)
            }
            Self::SetTermSize(e) => {
                write!(f, "error setting terminal size: {}", e)
            }
            Self::Spawn(e) => {
                write!(f, "error spawning subprocess: {}", e)
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CreatePty(e) | Self::SetTermSize(e) | Self::Spawn(e) => {
                match e {
                    Source::Io(e) => Some(e),
                    Source::Nix(e) => Some(e),
                }
            }
        }
    }
}

/// Convenience wrapper for `Result`s using [`Error`](Error)
pub type Result<T> = std::result::Result<T, Error>;

pub fn create_pty<S: std::convert::Into<Source>>(source: S) -> Error {
    Error::CreatePty(source.into())
}

pub fn set_term_size<S: std::convert::Into<Source>>(source: S) -> Error {
    Error::SetTermSize(source.into())
}

pub fn spawn<S: std::convert::Into<Source>>(source: S) -> Error {
    Error::Spawn(source.into())
}
