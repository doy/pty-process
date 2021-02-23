use std::os::unix::io::FromRawFd as _;
use std::os::unix::process::CommandExt as _;

impl super::CommandImpl for std::process::Command {
    type Child = std::process::Child;

    fn std_fds(
        &mut self,
        stdin: ::std::os::unix::io::RawFd,
        stdout: ::std::os::unix::io::RawFd,
        stderr: ::std::os::unix::io::RawFd,
    ) {
        // safe because the fds are valid (otherwise pty.pts() or dup() would
        // have returned an Err and we would have exited early) and are not
        // owned by any other structure (since dup() returns a fresh copy of
        // the file descriptor), allowing from_raw_fd to take ownership of it.
        self.stdin(unsafe { std::process::Stdio::from_raw_fd(stdin) })
            .stdout(unsafe { std::process::Stdio::from_raw_fd(stdout) })
            .stderr(unsafe { std::process::Stdio::from_raw_fd(stderr) });
    }

    fn pre_exec_impl<F>(&mut self, f: F)
    where
        F: FnMut() -> ::std::io::Result<()> + Send + Sync + 'static,
    {
        // safe because setsid() and close() are async-signal-safe functions
        // and ioctl() is a raw syscall (which is inherently
        // async-signal-safe).
        unsafe {
            self.pre_exec(f);
        }
    }

    fn spawn_impl(&mut self) -> ::std::io::Result<Self::Child> {
        self.spawn()
    }
}
