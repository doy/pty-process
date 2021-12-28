type Pt = crate::pty::tokio::AsyncPty;

pub type Command = crate::Command<tokio::process::Command, Pt>;
pub type Child = crate::Child<tokio::process::Child, Pt>;
pub type Pty = crate::Pty<Pt>;
