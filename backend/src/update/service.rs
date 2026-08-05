pub fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
