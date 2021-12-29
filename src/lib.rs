#![warn(clippy::cargo)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::as_conversions)]
#![warn(clippy::get_unwrap)]
#![allow(clippy::cognitive_complexity)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::similar_names)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::type_complexity)]

mod error;
pub use error::{Error, Result, Source};
mod types;
pub use types::Size;

mod sys;

pub mod blocking;

#[cfg(feature = "async")]
mod command;
#[cfg(feature = "async")]
pub use command::{Child, Command};
#[cfg(feature = "async")]
mod pty;
#[cfg(feature = "async")]
pub use pty::Pty;
