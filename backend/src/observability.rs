pub fn init() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("glucose_dashboard=info")
        .try_init();
}
