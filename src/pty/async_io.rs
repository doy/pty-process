impl super::Impl for async_io::Async<std::fs::File> {
    fn new_from_fh(fh: std::fs::File) -> crate::Result<Self> {
        Self::new(fh).map_err(crate::error::create_pty)
    }
}
