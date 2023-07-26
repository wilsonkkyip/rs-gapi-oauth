use serde::Deserialize;
use serde::Serialize;

pub const OAUTH_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

#[derive(Debug, Deserialize, Serialize)]
pub struct Token {
    pub access_token: String,
    pub expires_in: i64
}