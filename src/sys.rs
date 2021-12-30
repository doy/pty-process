use std::os::unix::io::{FromRawFd as _, IntoRawFd as _};

pub fn create_pt() -> nix::Result<(std::fs::File, std::path::PathBuf)> {
    let pt = nix::pty::posix_openpt(
        nix::fcntl::OFlag::O_RDWR
            | nix::fcntl::OFlag::O_NOCTTY
            | nix::fcntl::OFlag::O_CLOEXEC,
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
    pts: &impl std::os::unix::io::AsRawFd,
) -> nix::Result<(
    std::process::Stdio,
    std::process::Stdio,
    std::process::Stdio,
)> {
    let pts_fd = pts.as_raw_fd();

    let stdin =
        nix::fcntl::fcntl(pts_fd, nix::fcntl::FcntlArg::F_DUPFD_CLOEXEC(0))?;
    let stdout =
        nix::fcntl::fcntl(pts_fd, nix::fcntl::FcntlArg::F_DUPFD_CLOEXEC(0))?;
    let stderr =
        nix::fcntl::fcntl(pts_fd, nix::fcntl::FcntlArg::F_DUPFD_CLOEXEC(0))?;

    // Safety: these file descriptors were all just returned from dup, so they
    // must be valid
    Ok((
        unsafe { std::process::Stdio::from_raw_fd(stdin) },
        unsafe { std::process::Stdio::from_raw_fd(stdout) },
        unsafe { std::process::Stdio::from_raw_fd(stderr) },
    ))
}

pub fn session_leader(
    pts: &impl std::os::unix::io::AsRawFd,
) -> impl FnMut() -> std::io::Result<()> {
    let pts_fd = pts.as_raw_fd();
    move || {
        nix::unistd::setsid()?;
        set_controlling_terminal(pts_fd)?;
        Ok(())
    }
}

pub fn set_process_group_child(
    pg: Option<u32>,
) -> impl FnMut() -> std::io::Result<()> {
    move || {
        nix::unistd::setpgid(
            nix::unistd::Pid::from_raw(0),
            pg.map_or(nix::unistd::Pid::from_raw(0), |pid| {
                nix::unistd::Pid::from_raw(pid.try_into().unwrap())
            }),
        )?;
        Ok(())
    }
}

pub fn set_process_group_parent(
    pid: u32,
    pg: Option<u32>,
) -> nix::Result<()> {
    nix::unistd::setpgid(
        nix::unistd::Pid::from_raw(pid.try_into().unwrap()),
        pg.map_or(nix::unistd::Pid::from_raw(0), |pid| {
            nix::unistd::Pid::from_raw(pid.try_into().unwrap())
        }),
    )
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
