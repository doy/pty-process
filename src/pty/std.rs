impl super::Impl for std::fs::File {
    fn new_from_fh(fh: std::fs::File) -> crate::Result<Self> {
        Ok(fh)
    }
}
