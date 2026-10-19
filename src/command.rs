/// Wrapper around [`tokio::process::Command`]
pub struct Command {
    inner: tokio::process::Command,
    stdin: bool,
    stdout: bool,
    stderr: bool,
    pre_exec_set: bool,
    pre_exec: Option<
        Box<dyn FnMut() -> std::io::Result<()> + Send + Sync + 'static>,
    >,
}

impl Command {
    /// See [`tokio::process::Command::new`]
    pub fn new<S: AsRef<std::ffi::OsStr>>(program: S) -> Self {
        tokio::process::Command::new(program).into()
    }

    /// See [`tokio::process::Command::arg`]
    #[must_use]
    pub fn arg<S: AsRef<std::ffi::OsStr>>(mut self, arg: S) -> Self {
        self.inner.arg(arg);
        self
    }

    /// See [`tokio::process::Command::args`]
    #[must_use]
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        self.inner.args(args);
        self
    }

    /// See [`tokio::process::Command::env`]
    #[must_use]
    pub fn env<K, V>(mut self, key: K, val: V) -> Self
    where
        K: AsRef<std::ffi::OsStr>,
        V: AsRef<std::ffi::OsStr>,
    {
        self.inner.env(key, val);
        self
    }

    /// See [`tokio::process::Command::envs`]
    #[must_use]
    pub fn envs<I, K, V>(mut self, vars: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<std::ffi::OsStr>,
        V: AsRef<std::ffi::OsStr>,
    {
        self.inner.envs(vars);
        self
    }

    /// See [`tokio::process::Command::env_remove`]
    #[must_use]
    pub fn env_remove<K: AsRef<std::ffi::OsStr>>(mut self, key: K) -> Self {
        self.inner.env_remove(key);
        self
    }

    /// See [`tokio::process::Command::env_clear`]
    #[must_use]
    pub fn env_clear(mut self) -> Self {
        self.inner.env_clear();
        self
    }

    /// See [`tokio::process::Command::current_dir`]
    #[must_use]
    pub fn current_dir<P: AsRef<std::path::Path>>(mut self, dir: P) -> Self {
        self.inner.current_dir(dir);
        self
    }

    /// See [`tokio::process::Command::kill_on_drop`]
    #[must_use]
    pub fn kill_on_drop(mut self, kill_on_drop: bool) -> Self {
        self.inner.kill_on_drop(kill_on_drop);
        self
    }

    /// See [`tokio::process::Command::stdin`]
    #[must_use]
    pub fn stdin<T: Into<std::process::Stdio>>(mut self, cfg: T) -> Self {
        self.stdin = true;
        self.inner.stdin(cfg);
        self
    }

    /// See [`tokio::process::Command::stdout`]
    #[must_use]
    pub fn stdout<T: Into<std::process::Stdio>>(mut self, cfg: T) -> Self {
        self.stdout = true;
        self.inner.stdout(cfg);
        self
    }

    /// See [`tokio::process::Command::stderr`]
    #[must_use]
    pub fn stderr<T: Into<std::process::Stdio>>(mut self, cfg: T) -> Self {
        self.stderr = true;
        self.inner.stderr(cfg);
        self
    }

    /// Executes the command as a child process via
    /// [`tokio::process::Command::spawn`] on the given pty. The pty will be
    /// attached to all of `stdin`, `stdout`, and `stderr` of the child,
    /// unless those file descriptors were previously overridden through calls
    /// to [`stdin`](Self::stdin), [`stdout`](Self::stdout), or
    /// [`stderr`](Self::stderr). The newly created child process will also be
    /// made the session leader of a new session, and will have the given
    /// pty set as its controlling terminal.
    ///
    /// # Errors
    /// Returns an error if we fail to allocate new file descriptors for
    /// attaching the pty to the child process, or if we fail to spawn the
    /// child process (see the documentation for
    /// [`tokio::process::Command::spawn`]), or if we fail to make the child a
    /// session leader or set its controlling terminal.
    #[allow(clippy::needless_pass_by_value)]
    pub fn spawn(
        mut self,
        pts: crate::Pts,
    ) -> crate::Result<tokio::process::Child> {
        self.spawn_impl(&pts)
    }

    /// Executes the command as a child process via
    /// [`tokio::process::Command::spawn`] on the given pty. The pty will be
    /// attached to all of `stdin`, `stdout`, and `stderr` of the child,
    /// unless those file descriptors were previously overridden through calls
    /// to [`stdin`](Self::stdin), [`stdout`](Self::stdout), or
    /// [`stderr`](Self::stderr). The newly created child process will also be
    /// made the session leader of a new session, and will have the given
    /// pty set as its controlling terminal.
    ///
    /// Differs from `spawn` in that it borrows the pty rather than consuming
    /// it, allowing for multiple commands to be spawned onto the same pty in
    /// sequence, but this functionality is not available on macos.
    ///
    /// # Errors
    /// Returns an error if we fail to allocate new file descriptors for
    /// attaching the pty to the child process, or if we fail to spawn the
    /// child process (see the documentation for
    /// [`tokio::process::Command::spawn`]), or if we fail to make the child a
    /// session leader or set its controlling terminal.
    #[cfg(not(target_os = "macos"))]
    pub fn spawn_borrowed(
        &mut self,
        pts: &crate::Pts,
    ) -> crate::Result<tokio::process::Child> {
        self.spawn_impl(pts)
    }

    fn spawn_impl(
        &mut self,
        pts: &crate::Pts,
    ) -> crate::Result<tokio::process::Child> {
        let (stdin, stdout, stderr) = pts.0.setup_subprocess()?;

        if !self.stdin {
            self.inner.stdin(stdin);
        }
        if !self.stdout {
            self.inner.stdout(stdout);
        }
        if !self.stderr {
            self.inner.stderr(stderr);
        }

        let mut session_leader = pts.0.session_leader();
        // Safety: setsid() is an async-signal-safe function and ioctl() is a
        // raw syscall (which is inherently async-signal-safe).
        if let Some(mut custom) = self.pre_exec.take() {
            unsafe {
                self.inner.pre_exec(move || {
                    session_leader()?;
                    custom()?;
                    Ok(())
                })
            };
        } else if !self.pre_exec_set {
            unsafe { self.inner.pre_exec(session_leader) };
        }
        self.pre_exec_set = true;

        Ok(self.inner.spawn()?)
    }

    /// See [`tokio::process::Command::uid`]
    #[must_use]
    pub fn uid(mut self, id: u32) -> Self {
        self.inner.uid(id);
        self
    }

    /// See [`tokio::process::Command::gid`]
    #[must_use]
    pub fn gid(mut self, id: u32) -> Self {
        self.inner.gid(id);
        self
    }

    /// See [`tokio::process::Command::pre_exec`]
    #[allow(clippy::missing_safety_doc)]
    #[must_use]
    pub unsafe fn pre_exec<F>(mut self, f: F) -> Self
    where
        F: FnMut() -> std::io::Result<()> + Send + Sync + 'static,
    {
        self.pre_exec = Some(Box::new(f));
        self
    }

    /// See [`tokio::process::Command::arg0`]
    #[must_use]
    pub fn arg0<S>(mut self, arg: S) -> Self
    where
        S: AsRef<std::ffi::OsStr>,
    {
        self.inner.arg0(arg);
        self
    }
}

impl From<tokio::process::Command> for Command {
    fn from(inner: tokio::process::Command) -> Self {
        Self {
            inner,
            stdin: false,
            stdout: false,
            stderr: false,
            pre_exec_set: false,
            pre_exec: None,
        }
    }
}
