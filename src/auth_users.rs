use std::fs;
use serde_json::Value;
use serde::Deserialize;
use serde::Serialize;
use urlencoding::encode;
use std::net::TcpListener;
use std::io::prelude::Read;
use url::Url;
use std::collections::HashMap;
use chrono::prelude::NaiveDateTime;
use crate::token::Token;
use crate::token::OAUTH_TOKEN_URL;

#[derive(Debug, Deserialize, Serialize)]
pub struct UserSecret {
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: String
}

impl UserSecret {
    #[allow(dead_code)]
    pub fn from_file(path: &str) -> Result<UserSecret, std::io::Error> {
        
        let binding: String = fs::read_to_string(path)?;
        
        let content = serde_json::from_str(&binding.as_str())
            .expect("Failed to parse file to UserSecret");

        return Ok(content);
    }

    pub async fn auth(&self) -> Result<Token, reqwest::Error> {
        // Prepare auth body
        let mut body: Value = serde_json::to_value(&self)
            .expect("Could not convert UserSecret to Value");
        body["grant_type"] = Value::String("refresh_token".to_string());

        // Auth request
        let response: reqwest::Response = reqwest::Client::new()
            .post(OAUTH_TOKEN_URL)
            .json(&body)
            .send()
            .await?;

        // Parse response to output
        let content: Token = response.json().await?;

        return Ok(content)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientSecret {
    pub client_id: String,
    pub project_id: String,
    pub auth_uri: String,
    pub token_uri: String,
    pub auth_provider_x509_cert_url: String,
    pub client_secret: String,
    pub redirect_uris: Vec<String>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthCode {
    pub code: String,
    pub scope: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientSecretTokenResponse {
    pub access_token: String,
    pub expires_in: i64,
    pub refresh_token: String,
    pub scope: String,
    pub token_type: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientSerectResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub scopes: Vec<String>,
    pub expiry: String,
    pub auth_uri: String,
    pub token_uri: String,
    pub client_id: String,
    pub client_secret: String
}

impl ClientSecret {
    #[allow(dead_code)]
    pub fn from_file(path: &str) -> Result<ClientSecret, std::io::Error> {
        let bindings: String = fs::read_to_string(&path)?;
        let content: Value = serde_json::from_str(&bindings.as_str())
            .expect("Failed to parse file to JSON");
        let content: String = serde_json::to_string(&content["web"])
            .expect("No `web` section in ClientSecret");
        let content: ClientSecret = serde_json::from_str(&content.as_str())
            .expect("Cannot parse to ClientSecret");

        Ok(content)
    }

    #[allow(dead_code)]
    pub fn auth_url(&self, scope: &str) -> String {
        let url: String = format!(
            "{}?client_id={}&redirect_uri={}&scope={}&access_type=offline&prompt=consent&response_type=code",
            self.auth_uri,
            encode(&self.client_id),
            encode(&self.redirect_uris[0]),
            encode(scope)
        );
        return url;
    }

    #[allow(dead_code)]
    pub fn auth_code(&self, scope: &str, port: u32) -> Result<AuthCode, std::io::Error> {
        let auth_url: String = self.auth_url(scope);
        println!("Please visit this URL to authorize this application: {}", auth_url);

        let listener: TcpListener = 
            TcpListener::bind(format!("localhost:{}", port))
                .expect("Failed to bind to port");
        
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0;2048];
        stream.read(&mut buf).unwrap();

        let buf_str: String = String::from_utf8_lossy(&buf[..]).to_string();
        let buf_vec: Vec<&str> = buf_str
            .split(" ")
            .collect::<Vec<&str>>();

        let args: String = buf_vec[1].to_string();
        let callback_url: Url = Url::parse(
            (format!("http://localhost:{}", port) + &args).as_str()
        ).expect("Failed to parse callback URL");
        let query: HashMap<_,_> = callback_url.query_pairs().into_owned().collect();
        let output = AuthCode {
            code: query.get("code").unwrap().to_string(),
            scope: query.get("scope").unwrap().to_string()
        };
        return Ok(output);
    }

    #[allow(dead_code)]
    pub async fn auth_token(&self, code: &str) -> Result<ClientSecretTokenResponse, reqwest::Error> {
        let body: Value = serde_json::json!({
            "client_id": self.client_id,
            "client_secret": self.client_secret,
            "code": code,
            "grant_type": "authorization_code",
            "redirect_uri": self.redirect_uris[0]
        });

        let response = reqwest::Client::new()
            .post(self.token_uri.as_str())
            .json(&body)
            .send()
            .await?;

        let content: ClientSecretTokenResponse = response.json().await.expect("Failed to parse http response");

        return Ok(content);
    }

    #[allow(dead_code)]
    pub async fn auth(&self, scope: &str, port: u32) -> Result<ClientSerectResponse, reqwest::Error> {
        let auth_code = self.auth_code(&scope, port)
            .expect("Failed to parse auth code");

        let ts = chrono::Utc::now().timestamp_micros();

        let user_token: ClientSecretTokenResponse = self.auth_token(&auth_code.code).await?;

        let expiry_int = 3599 * 1000000 + ts;
        let expiry_naive = NaiveDateTime::from_timestamp_micros(expiry_int).unwrap();

        let scopes: Vec<&str> = scope.split(" ").collect::<Vec<&str>>();
        let scopes: Vec<String> = scopes.iter().map(|s| s.to_string())
            .collect::<Vec<String>>();

        let output: ClientSerectResponse = ClientSerectResponse{
            access_token: user_token.access_token,
            refresh_token: user_token.refresh_token,
            scopes: scopes,
            expiry: expiry_naive.format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string(),
            auth_uri: self.auth_uri.clone(),
            token_uri: self.token_uri.clone(),
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.clone()
        };
        
        return Ok(output);
    }
}
