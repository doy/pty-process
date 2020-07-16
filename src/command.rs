use crate::error::*;

use std::os::unix::io::{AsRawFd as _, FromRawFd as _};
use std::os::unix::process::CommandExt as _;

pub struct Command {
    command: std::process::Command,
}

impl Command {
    pub fn new<S: AsRef<std::ffi::OsStr>>(program: S) -> Result<Self> {
        let command = std::process::Command::new(program);
        Ok(Self { command })
    }

    pub fn arg<S: AsRef<std::ffi::OsStr>>(&mut self, arg: S) -> &mut Self {
        self.command.arg(arg);
        self
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        self.command.args(args);
        self
    }

    pub fn env<K, V>(&mut self, key: K, val: V) -> &mut Self
    where
        K: AsRef<std::ffi::OsStr>,
        V: AsRef<std::ffi::OsStr>,
    {
        self.command.env(key, val);
        self
    }

    pub fn envs<I, K, V>(&mut self, vars: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<std::ffi::OsStr>,
        V: AsRef<std::ffi::OsStr>,
    {
        self.command.envs(vars);
        self
    }

    pub fn env_remove<K: AsRef<std::ffi::OsStr>>(
        &mut self,
        key: K,
    ) -> &mut Self {
        self.command.env_remove(key);
        self
    }

    pub fn env_clear(&mut self) -> &mut Self {
        self.command.env_clear();
        self
    }

    pub fn current_dir<P: AsRef<std::path::Path>>(
        &mut self,
        dir: P,
    ) -> &mut Self {
        self.command.current_dir(dir);
        self
    }

    pub fn uid(&mut self, id: u32) -> &mut Self {
        self.command.uid(id);
        self
    }

    pub fn gid(&mut self, id: u32) -> &mut Self {
        self.command.gid(id);
        self
    }

    pub fn spawn(&mut self) -> Result<Child> {
        let pty = crate::pty::Pty::new()?;
        let pt_fd = pty.pt().as_raw_fd();
        let pts = pty.pts()?;
        let pts_fd = pts.as_raw_fd();
        self.command
            .stdin(unsafe { std::process::Stdio::from_raw_fd(pts_fd) })
            .stdout(unsafe { std::process::Stdio::from_raw_fd(pts_fd) })
            .stderr(unsafe { std::process::Stdio::from_raw_fd(pts_fd) });
        unsafe {
            self.command.pre_exec(move || {
                // XXX unwrap
                nix::unistd::close(pt_fd)
                    .map_err(|e| e.as_errno().unwrap())?;
                nix::unistd::close(pts_fd)
                    .map_err(|e| e.as_errno().unwrap())?;
                Ok(())
            });
        }
        let child = self.command.spawn()?;
        Ok(Child { child, pty })
    }
}

pub struct Child {
    child: std::process::Child,
    pty: crate::pty::Pty,
}

impl Child {
    pub fn pty(&self) -> &std::fs::File {
        self.pty.pt()
    }
}

impl std::ops::Deref for Child {
    type Target = std::process::Child;

    fn deref(&self) -> &Self::Target {
        &self.child
    }
}

impl std::ops::DerefMut for Child {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.child
    }
}
