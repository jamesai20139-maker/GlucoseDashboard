#[derive(Clone, Default)]
pub struct CredentialStore;

impl CredentialStore {
    pub fn available(&self) -> bool {
        cfg!(windows) || std::env::var("GLUCOSE_ALLOW_INSECURE_DEV_AUTH").is_ok()
    }
    pub fn status(&self) -> &'static str {
        if self.available() {
            "available"
        } else {
            "unavailable"
        }
    }
}
