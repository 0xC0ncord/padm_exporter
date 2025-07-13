use serde::Deserialize;

#[derive(Deserialize)]
pub struct AuthData {
    pub access_token: String,
    pub refresh_token: String,
    pub msg: String,
}
impl AuthData {
    pub fn new() -> AuthData {
        AuthData {
            access_token: String::new(),
            refresh_token: String::new(),
            msg: String::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.access_token.is_empty() || self.refresh_token.is_empty() || self.msg.is_empty()
    }
}
