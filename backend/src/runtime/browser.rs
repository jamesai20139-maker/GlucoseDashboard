pub fn open_dashboard(url: &str) -> Result<(), String> {
    open::that(url).map_err(|error| error.to_string())
}
