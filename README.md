# gapi-oauth

A personal project for Google API OAuth 2.0. It only focuses on ["Sever-side Web Apps"](https://developers.google.com/identity/protocols/oauth2/web-server) and [Service Accounts](https://developers.google.com/identity/protocols/oauth2/service-account). It is designed to be used as both a CLI tool and a crate of package. 

## CLI Tool

```
Usage: gapi-oauth <SERVICE> <JSON_PATH> [SCOPE] [PORT]

SERVICE: `user`, `service`, or `consent`
JSON_PATH: The path to the JSON file containing the credentials.
SCOPE: Only required for `service` and `consent`
PORT: Only required for `consent`


Example:
gapi-oauth user /path/to/client_token.json
# {
#   "access_token": "ya29....",
#   "expires_in": 3599
# }

gapi-oauth service /path/to/service_account.json 'https://www.googleapis.com/auth/drive https://www.googleapis.com/auth/youtube'
# {
#   "access_token": "ya29....",
#   "expires_in": 3599
# }

gapi-oauth consent /path/to/client_secret.json 'https://www.googleapis.com/auth/drive https://www.googleapis.com/auth/youtube' 8088
# {
#   "access_token": "ya29.....",
#   "refresh_token": ".....",
#   "scopes": [
#     "https://www.googleapis.com/auth/drive",
#     "https://www.googleapis.com/auth/youtube"
#   ],
#   "expiry": "2023-07-26T18:30:28.123456Z",
#   "auth_uri": "https://accounts.google.com/o/oauth2/auth",
#   "token_uri": "https://oauth2.googleapis.com/token",
#   "client_id": "......",
#   "client_secret": "......"
# }
```

## Cargo crate

### Usage
Below shows an example for authenticating an `cleint_token.json` file with `refresh_token` inside. 
```rust
use gapi_oauth::{UserSecret, UserToken};
use serde_json;

#[tokio::main]
async fn main() {
    let fpath: &str = "/path/to/client_token.json";
    let user_secret: UserSecret = UserSecret::from_file(&fpath);
    let token: UserToken = user_secret.auth().await.unwrap();
    let token_str: String = serde_json::to_string_pretty(&token).unwrap();

    println!("{}", token_str);
    // {
    //   "access_token": "ya29....",
    //   "expires_in": 3599
    // }
}
```

### Structs

**Token**
```rust
pub struct Token {
    pub access_token: String,
    pub expires_in: i64
}
```

**UserSecret**
```rust
pub struct UserSecret {
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: String
}

impl UserSecret {
    pub fn from_file(path: &str) -> Result<UserSecret, std::io::Error> 
    pub async fn auth(&self) -> Result<Token, reqwest::Error>
}
```

**ServiceSecret**
```rust
pub struct ServiceSecret {
    pub client_email: String,
    pub private_key_id: String,
    pub private_key: String
}

impl ServiceSecret {
    pub fn from_file(path: &str) -> Result<ServiceSecret, std::io::Error>
    pub async fn auth(&self, scope: &str) -> Result<Token, reqwest::Error>
}
```

**ClientSecret**
```rust
pub struct ClientSecret {
    pub client_id: String,
    pub project_id: String,
    pub auth_uri: String,
    pub token_uri: String,
    pub auth_provider_x509_cert_url: String,
    pub client_secret: String,
    pub redirect_uris: Vec<String>
}

pub struct AuthCode {
    pub code: String,
    pub scope: String
}

pub struct ClientSecretTokenResponse {
    pub access_token: String,
    pub expires_in: i64,
    pub refresh_token: String,
    pub scope: String,
    pub token_type: String
}

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
    pub fn from_file(path: &str) -> Result<ClientSecret, std::io::Error>
    pub fn auth_code(&self, scope: &str, port: u32) -> Result<AuthCode, std::io::Error>
    pub async fn auth_token(&self, code: &str) -> Result<ClientSecretTokenResponse, reqwest::Error>
    pub async fn auth(&self, scope: &str, port: u32) -> Result<ClientSerectResponse, reqwest::Error>
}
```

