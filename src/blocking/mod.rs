//! Blocking equivalents for [`pty_process::Command`](crate::Command) and
//! [`pty_process::Pty`](crate::Pty)

mod command;
pub use command::Command;
mod pty;
pub use pty::{Pts, Pty};
