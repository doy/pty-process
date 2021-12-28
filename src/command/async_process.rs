use async_process::unix::CommandExt as _;

impl super::Impl for async_process::Command {
    type Child = async_process::Child;

    fn new_impl(program: &::std::ffi::OsStr) -> Self {
        Self::new(program)
    }

    fn stdin_impl(&mut self, cfg: ::std::process::Stdio) {
        self.stdin(cfg);
    }

    fn stdout_impl(&mut self, cfg: ::std::process::Stdio) {
        self.stdout(cfg);
    }

    fn stderr_impl(&mut self, cfg: ::std::process::Stdio) {
        self.stderr(cfg);
    }

    unsafe fn pre_exec_impl<F>(&mut self, f: F)
    where
        F: FnMut() -> ::std::io::Result<()> + Send + Sync + 'static,
    {
        self.pre_exec(f);
    }

    fn spawn_impl(&mut self) -> crate::Result<Self::Child> {
        self.spawn().map_err(crate::error::spawn)
    }
}
