#[path = "token.rs"]
mod token;

use std::fs;
use serde::Deserialize;
use serde::Serialize;
pub use token::Token;

use jsonwebtoken::{
    encode, 
    Algorithm,
    Header,
    EncodingKey
};

use reqwest::Response;
use reqwest::StatusCode;


#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceSecret {
    pub client_email: String,
    pub private_key_id: String,
    pub private_key: String
}

impl ServiceSecret {
    pub fn from_file(path: &str) -> Result<ServiceSecret, std::io::Error> {
        let bindings = fs::read_to_string(&path)?;
        let content = serde_json::from_str(&bindings.as_str())
            .expect("Unable to parse file to ServiceSecret");
        return Ok(content);
    }

    pub async fn auth(&self, scope: &str) -> Result<Token, reqwest::Error> {
        // Auth Service Account
        // https://developers.google.com/identity/protocols/oauth2/service-account

        // Prepare JWT claim
        let claim: serde_json::Value = serde_json::json!({
            "iss": self.client_email.to_string(),
            "scope": scope.to_string(),
            "aud": "https://oauth2.googleapis.com/token".to_string(),
            "iat": chrono::offset::Utc::now().timestamp(),
            "exp": chrono::offset::Utc::now().timestamp() + 3600
        });

        // Prepare JWT header
        let mut header: Header = Header::new(Algorithm::RS256);
        header.kid = Some(self.private_key_id.to_string());

        // Prepare JWT key
        let key: EncodingKey = EncodingKey::from_rsa_pem(
            &self.private_key
                .to_string()
                .replace("\\n", "\n").as_bytes()
        ).expect("Cannot build `EncodingKey`.");

        // Generate JWT
        let token: String = encode(
            &header, &claim, &key
        ).expect("Cannot encode `token`.");

        // Auth JWT
        let response: Response = reqwest::Client::new()
            .post(token::OAUTH_TOKEN_URL)
            .json(&serde_json::json!({
                "grant_type": "urn:ietf:params:oauth:grant-type:jwt-bearer",
                "assertion": token
            }))
            .send()
            .await?;
        
        // Prepare output
        let content: Token = match response.status() {
            StatusCode::OK => response.json().await.expect("Unable to parse HTTP response JSON."),
            StatusCode::UNAUTHORIZED => {
                println!("{}", response.text().await.unwrap());
                panic!("HTTP request failed: Unauthorized.");
            },
            _ => {
                println!("{}", response.text().await.unwrap());
                panic!("HTTP request failed.");
            }
        };

        return Ok(content);
    }
}
