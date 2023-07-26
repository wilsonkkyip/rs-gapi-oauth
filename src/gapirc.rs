#[path = "auth_users.rs"] 
mod auth_users;
use auth_users::UserSecret;
#[path = "auth_service.rs"] 
mod auth_service;
use auth_service::ServiceSecret;
use serde::{Deserialize, Serialize};
use chrono;
use chrono::prelude::NaiveDateTime;



#[derive(Debug, Deserialize, Serialize)]
pub struct GApiRc {
    pub auth_type: Option<GAuthType>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub refresh_token: Option<String>,
    pub scopes: Option<Vec<String>>,
    pub client_email: Option<String>,
    pub private_key_id: Option<String>,
    pub private_key: Option<String>,
    pub access_token: Option<String>,
    pub expiry: Option<String>
}

#[derive(Debug, Deserialize, Serialize)]
pub enum GAuthType {
    #[serde(rename = "oauth_user")]
    OauthUser,
    #[serde(rename = "oauth_service")]
    OauthService
}

impl GApiRc {
    fn _verify(&mut self) {
        if self.auth_type.is_none() {
            let client_id = &self.client_id;
            let client_secret = &self.client_secret;
            let refresh_token = &self.refresh_token;
            let scopes = &self.scopes;
            let client_email = &self.client_email;
            let private_key_id = &self.private_key_id;
            let private_key = &self.private_key;

            if client_id.is_some() && client_secret.is_some() && refresh_token.is_some() && scopes.is_some() {
                self.auth_type = Some(GAuthType::OauthUser);
            }

            if client_email.is_some() && private_key_id.is_some() && private_key.is_some() {
                self.auth_type = Some(GAuthType::OauthService);
            }
        }
    }

    pub fn from_file(path: &str) -> Result<GApiRc, std::io::Error> {
        let bindings = std::fs::read_to_string(&path)?;
        let mut content: GApiRc = serde_json::from_str(&bindings.as_str())
            .expect("Failed to parse file to GApiRc");
        content._verify();
        return Ok(content);
    }

    pub async fn auth(&mut self) {
        let ts = chrono::Utc::now().timestamp_micros();

        match self.auth_type.as_ref().expect("auth_type is None") {
            GAuthType::OauthUser => {
                let user_secret: UserSecret = UserSecret {
                    client_id: self.client_id.clone().expect("client_id is None"),
                    client_secret: self.client_secret.clone().expect("client_secret is None"),
                    refresh_token: self.refresh_token.clone().expect("refresh_token is None")
                };
                let token = user_secret.auth().await
                    .expect("Failed to auth UserSecret");

                let expiry_int = token.expires_in * 1000000 + ts;
                let expiry_naive = NaiveDateTime::from_timestamp_micros(expiry_int).unwrap();
                self.expiry = Some(expiry_naive.format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string());
                self.access_token = Some(token.access_token);
            }
            GAuthType::OauthService => {
                let service_secret: ServiceSecret = ServiceSecret {
                    client_email: self.client_email.clone().expect("client_email is None"),
                    private_key_id: self.private_key_id.clone().expect("private_key_id is None"),
                    private_key: self.private_key.clone().expect("private_key is None")
                };
                let scopes = &self.scopes.as_ref()
                    .expect("No scopes in GApiRc");

                let scope = scopes.join(" ");
                let token = service_secret.auth(&scope).await
                    .expect("Failed to auth ServiceSecret");

                let expiry_int = token.expires_in * 1000000 + ts;
                let expiry_naive = NaiveDateTime::from_timestamp_micros(expiry_int).unwrap();
                self.expiry = Some(expiry_naive.format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string());
                self.access_token = Some(token.access_token);
            }
        }
    }

    pub fn expired(&self) -> bool {
        let ts: i64 = chrono::Utc::now().timestamp_micros();

        if self.expiry.is_none() {
            panic!("No expiry in GApiRc");
        }
        
        let expiry = NaiveDateTime::parse_from_str(
            self.expiry.as_ref().unwrap(), 
            "%Y-%m-%dT%H:%M:%S%.6f%Z"
        ).unwrap().timestamp_micros();
        
        return ts > expiry;
    }

    pub fn write_rc(&self) {
        let home: String = std::env::var("HOME").unwrap();
        let path: String = format!("{home}/.gapirc.json");
        std::fs::write(path, serde_json::to_string_pretty(self).unwrap()).unwrap();
    }
}


