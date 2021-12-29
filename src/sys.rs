use std::os::unix::io::{FromRawFd as _, IntoRawFd as _};

pub fn create_pt() -> nix::Result<(std::fs::File, std::path::PathBuf)> {
    let pt = nix::pty::posix_openpt(
        nix::fcntl::OFlag::O_RDWR | nix::fcntl::OFlag::O_NOCTTY,
    )?;
    nix::pty::grantpt(&pt)?;
    nix::pty::unlockpt(&pt)?;

    let ptsname = nix::pty::ptsname_r(&pt)?.into();

    let pt_fd = pt.into_raw_fd();

    // Safety: posix_openpt (or the previous functions operating on the
    // result) would have returned an Err (causing us to return early) if the
    // file descriptor was invalid. additionally, into_raw_fd gives up
    // ownership over the file descriptor, allowing the newly created File
    // object to take full ownership.
    let pt = unsafe { std::fs::File::from_raw_fd(pt_fd) };

    Ok((pt, ptsname))
}

pub fn set_term_size(
    fh: &impl std::os::unix::io::AsRawFd,
    size: crate::Size,
) -> nix::Result<()> {
    let size = size.into();
    let fd = fh.as_raw_fd();

    // Safety: std::fs::File is required to contain a valid file descriptor
    // and size is guaranteed to be initialized because it's a normal rust
    // value, and nix::pty::Winsize is a repr(C) struct with the same layout
    // as `struct winsize` from sys/ioctl.h.
    unsafe {
        set_term_size_unsafe(fd, std::ptr::NonNull::from(&size).as_ptr())
    }
    .map(|_| ())
}

pub fn setup_subprocess(
    pt: &impl std::os::unix::io::AsRawFd,
    pts: impl std::os::unix::io::IntoRawFd,
) -> nix::Result<(
    std::process::Stdio,
    std::process::Stdio,
    std::process::Stdio,
    impl FnMut() -> std::io::Result<()>,
)> {
    let pt_fd = pt.as_raw_fd();
    let pts_fd = pts.into_raw_fd();

    let stdin = nix::unistd::dup(pts_fd)?;
    let stdout = nix::unistd::dup(pts_fd)?;
    let stderr = nix::unistd::dup(pts_fd)?;

    // Safety: these file descriptors were all just returned from dup, so they
    // must be valid
    Ok((
        unsafe { std::process::Stdio::from_raw_fd(stdin) },
        unsafe { std::process::Stdio::from_raw_fd(stdout) },
        unsafe { std::process::Stdio::from_raw_fd(stderr) },
        move || {
            nix::unistd::setsid()?;
            set_controlling_terminal(pts_fd)?;

            // in the parent, destructors will handle closing these file
            // descriptors (other than pt, used by the parent to
            // communicate with the child) when the function ends, but in
            // the child, we end by calling exec(), which doesn't call
            // destructors.

            nix::unistd::close(pt_fd)?;
            nix::unistd::close(pts_fd)?;
            // at this point, stdin/stdout/stderr have already been
            // reopened as fds 0/1/2 in the child, so we can (and should)
            // close the originals
            nix::unistd::close(stdin)?;
            nix::unistd::close(stdout)?;
            nix::unistd::close(stderr)?;

            Ok(())
        },
    ))
}

fn set_controlling_terminal(fd: std::os::unix::io::RawFd) -> nix::Result<()> {
    // Safety: std::fs::File is required to contain a valid file descriptor
    unsafe { set_controlling_terminal_unsafe(fd, std::ptr::null()) }
        .map(|_| ())
}

nix::ioctl_write_ptr_bad!(
    set_term_size_unsafe,
    libc::TIOCSWINSZ,
    nix::pty::Winsize
);

nix::ioctl_write_ptr_bad!(
    set_controlling_terminal_unsafe,
    libc::TIOCSCTTY,
    libc::c_int
);
