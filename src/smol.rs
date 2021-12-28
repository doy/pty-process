type Pt = async_io::Async<std::fs::File>;

pub type Command = crate::Command<async_process::Command, Pt>;
pub type Child = crate::Child<async_process::Child, Pt>;
pub type Pty = crate::Pty<Pt>;
