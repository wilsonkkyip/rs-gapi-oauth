use std::fs;
use serde::Deserialize;
use serde::Serialize;

use jsonwebtoken::{
    encode, 
    Algorithm,
    Header,
    EncodingKey
};

use reqwest::Response;
use reqwest::StatusCode;

use crate::token::Token;
use crate::token::OAUTH_TOKEN_URL;


#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceSecret {
    pub client_email: String,
    pub private_key_id: String,
    pub private_key: String
}

impl ServiceSecret {
    #[allow(dead_code)]
    pub fn from_file(path: &str) -> Result<ServiceSecret, std::io::Error> {
        let bindings: String = fs::read_to_string(&path)?;
        let content: ServiceSecret = serde_json::from_str(&bindings.as_str())
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
        let header: Header = Header{
            alg: Algorithm::RS256,
            kid: Some(self.private_key_id.to_string()),
            ..Default::default()
        };

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
            .post(OAUTH_TOKEN_URL)
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
