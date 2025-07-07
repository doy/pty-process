# Changelog

## [0.5.2] - 2025-07-07

### Fixed

* make documentation for async code show up on docs.rs

## [0.5.1] - 2025-01-29

### Added

* `as_fd` and `as_raw_fd` for `Pts` instances

## [0.5.0] - 2025-01-29

### Added

* `kill_on_drop` to match the tokio::process::Command behavior. (Samuel
  Ainsworth, #11)
* `from_fd` to unsafely create a Pty from an OwnedFd. (YtvwlD, #12)

### Changed

* Changed the `Command` builder API slightly to be harder to misuse on
  platforms (such as macos) which require opening a pts before doing any
  operations on the pty, and which don't support spawning more than one
  process onto a pts.

### Fixed

* macos should be better supported now.
* Spawning a process without an existing controlling terminal should now work.
  (Chris Pick, #16)

## [0.4.0] - 2023-08-06

### Changed

* Switch from nix to rustix, for hopefully better portability

### Added

* Implemented AsRawFd for the Pty structs

## [0.3.0] - 2023-03-08

### Changed

* Complete rewrite of the API
* Tokio is now the only supported backend, enabled via the `async` feature

## [0.2.0] - 2021-12-15

### Changed

* Simplified the `Error` type to remove a bunch of unnecessary distinctions

## [0.1.1] - 2021-11-10

### Changed

* Bumped deps and moved to 2021 edition

## [0.1.0] - 2021-03-06

### Added

* Initial release
