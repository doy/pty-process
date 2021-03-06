# pty-process

This crate adds a helper method to `std::process::Command` (and optionally its
equivalent in various async frameworks) to allocate a pty to spawn the process
into. This allows for manipulation of interactive programs.

The basic functionality is provided by the `Command` trait in this crate:

```rust
use pty_process::Command as _;

let mut cmd = std::process::Command::new("nethack");
let child = cmd.spawn_pty(Some(&pty_process::Size::new(24, 80))).unwrap();
```

The `child` instance returned by the call to `spawn_pty` is a thin wrapper
around the `Child` struct associated with the `Command` module used. You can
use it identically to how you would use the normal `std::process::Child`
instance, but it also provides additional methods for interacting with the pty:

```rust
use std::io::Write as _;

child.pty().write_all(b"foo\n").unwrap();
child.resize_pty(&pty_process::Size::new(30, 100)).unwrap();
let status = child.wait().unwrap();
```

The available implementations are gated by features:
* `backend-std`: Add an implementation for `std::process::Command`. Enabled by
  default.
* `backend-smol`: Add an implementation for `smol::process::Command`.
* `backend-async-std`: Add an implementation for `async_std::process::Command`.
* `backend-tokio`: Add an implementation for `tokio::process::Command`.

Any number of backends may be enabled, depending on your needs.
