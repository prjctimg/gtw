use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub client_id: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone)]
pub struct AuthSession {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: i64,
}

pub struct AuthManager;

impl AuthManager {
    pub fn new() -> Self {
        Self
    }

    pub fn pkce_challenge(&self) -> (String, String) {
        let verifier = uuid::Uuid::new_v4().to_string().replace('-', "");
        let challenge = verifier.clone();
        (verifier, challenge)
    }

    pub fn login_bot_password(&self, _username: &str, _password: &str) -> Option<AuthSession> {
        None
    }

    pub fn logout(&self) {}
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}
