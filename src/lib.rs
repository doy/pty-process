//! This crate is a wrapper around [`std::process::Command`] or
//! [`async_process::Command`] which provides the ability to allocate a pty
//! and spawn new processes attached to that pty, with the pty as their
//! controlling terminal. This allows for manipulation of interactive
//! programs.
//!
//! The basic functionality looks like this:
//!
//! ```no_run
//! let mut pty = pty_process::Pty::new().unwrap();
//! pty.resize(pty_process::Size::new(24, 80)).unwrap();
//! let mut cmd = pty_process::Command::new("nethack");
//! let child = cmd.spawn(&pty).unwrap();
//! ```
//!
//! The returned `child` is a normal instance of [`async_process::Child`] (or
//! [`std::process::Child`] for the [`blocking`](crate::blocking) variant),
//! with its `stdin`/`stdout`/`stderr` file descriptors pointing at the given
//! pty. The `pty` instance implements [`std::io::Read`] and
//! [`std::io::Write`] (or [`futures_io::AsyncRead`] and
//! [`futures_io::AsyncWrite`] for the [`blocking`] variant), and can be used
//! to communicate with the child process. The child process will also be made
//! a session leader of a new session, and the controlling terminal of that
//! session will be set to the given pty.
//!
//! # Features
//!
//! By default, only the [`blocking`](crate::blocking) APIs are available. To
//! include the asynchronous APIs, you must enable the `async` feature. See
//! the `examples` directory in the repository for examples of how to use this
//! crate with the various different asynchronous frameworks.

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
pub use error::{Error, Result};
mod types;
pub use types::Size;

mod sys;

pub mod blocking;

#[cfg(feature = "async")]
mod command;
#[cfg(feature = "async")]
pub use command::Command;
#[cfg(feature = "async")]
mod pty;
#[cfg(feature = "async")]
pub use pty::Pty;
