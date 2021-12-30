use async_process::unix::CommandExt as _;

pub struct Command {
    inner: async_process::Command,
    stdin: Option<std::process::Stdio>,
    stdout: Option<std::process::Stdio>,
    stderr: Option<std::process::Stdio>,
    pre_exec: Option<
        Box<dyn FnMut() -> std::io::Result<()> + Send + Sync + 'static>,
    >,
}

impl Command {
    pub fn new<S: AsRef<std::ffi::OsStr>>(program: S) -> Self {
        Self {
            inner: async_process::Command::new(program),
            stdin: None,
            stdout: None,
            stderr: None,
            pre_exec: None,
        }
    }

    pub fn arg<S: AsRef<std::ffi::OsStr>>(&mut self, arg: S) -> &mut Self {
        self.inner.arg(arg);
        self
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        self.inner.args(args);
        self
    }

    pub fn env<K, V>(&mut self, key: K, val: V) -> &mut Self
    where
        K: AsRef<std::ffi::OsStr>,
        V: AsRef<std::ffi::OsStr>,
    {
        self.inner.env(key, val);
        self
    }

    pub fn envs<I, K, V>(&mut self, vars: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<std::ffi::OsStr>,
        V: AsRef<std::ffi::OsStr>,
    {
        self.inner.envs(vars);
        self
    }

    pub fn env_remove<K: AsRef<std::ffi::OsStr>>(
        &mut self,
        key: K,
    ) -> &mut Self {
        self.inner.env_remove(key);
        self
    }

    pub fn env_clear(&mut self) -> &mut Self {
        self.inner.env_clear();
        self
    }

    pub fn current_dir<P: AsRef<std::path::Path>>(
        &mut self,
        dir: P,
    ) -> &mut Self {
        self.inner.current_dir(dir);
        self
    }

    pub fn stdin<T: Into<std::process::Stdio>>(
        &mut self,
        cfg: Option<T>,
    ) -> &mut Self {
        self.stdin = cfg.map(Into::into);
        self
    }

    pub fn stdout<T: Into<std::process::Stdio>>(
        &mut self,
        cfg: Option<T>,
    ) -> &mut Self {
        self.stdout = cfg.map(Into::into);
        self
    }

    pub fn stderr<T: Into<std::process::Stdio>>(
        &mut self,
        cfg: Option<T>,
    ) -> &mut Self {
        self.stderr = cfg.map(Into::into);
        self
    }

    pub fn spawn(
        &mut self,
        pty: &crate::Pty,
    ) -> crate::Result<async_process::Child> {
        let pts = pty.pts();
        let (stdin, stdout, stderr) = crate::sys::setup_subprocess(pts)?;

        self.inner.stdin(self.stdin.take().unwrap_or(stdin));
        self.inner.stdout(self.stdout.take().unwrap_or(stdout));
        self.inner.stderr(self.stderr.take().unwrap_or(stderr));

        let mut session_leader = crate::sys::session_leader(pts);
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
        } else {
            unsafe { self.inner.pre_exec(session_leader) };
        }

        Ok(self.inner.spawn()?)
    }

    pub fn spawn_pg(
        &mut self,
        pg: Option<u32>,
    ) -> crate::Result<async_process::Child> {
        let mut set_process_group = crate::sys::set_process_group_child(pg);
        // Safety: setpgid() is an async-signal-safe function and ioctl() is a
        // raw syscall (which is inherently async-signal-safe).
        if let Some(mut custom) = self.pre_exec.take() {
            unsafe {
                self.inner.pre_exec(move || {
                    set_process_group()?;
                    custom()?;
                    Ok(())
                })
            };
        } else {
            unsafe { self.inner.pre_exec(set_process_group) };
        }

        let child = self.inner.spawn()?;
        crate::sys::set_process_group_parent(child.id(), pg)?;
        Ok(child)
    }

    pub fn uid(&mut self, id: u32) -> &mut Self {
        self.inner.uid(id);
        self
    }

    pub fn gid(&mut self, id: u32) -> &mut Self {
        self.inner.gid(id);
        self
    }

    pub unsafe fn pre_exec<F>(&mut self, f: F) -> &mut Self
    where
        F: FnMut() -> std::io::Result<()> + Send + Sync + 'static,
    {
        self.pre_exec = Some(Box::new(f));
        self
    }

    pub fn arg0<S>(&mut self, arg: S) -> &mut Self
    where
        S: AsRef<std::ffi::OsStr>,
    {
        self.inner.arg0(arg);
        self
    }
}
