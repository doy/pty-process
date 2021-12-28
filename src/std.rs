type Pt = std::fs::File;

pub type Command = crate::Command<std::process::Command, Pt>;
pub type Child = crate::Child<std::process::Child, Pt>;
pub type Pty = crate::Pty<Pt>;
