use crate::auth_users::UserSecret;
use crate::auth_service::ServiceSecret;

#[tokio::test]
async fn test_auth_user() {
    let client_id = std::env::var("USER_CLIENT_ID")
        .expect("No USER_CLIENT_ID in env var")
        .as_str().to_string();
    let client_secret = std::env::var("USER_CLIENT_SECRET")
        .expect("No USER_CLIENT_SECRET in env var")
        .as_str().to_string();
    let refresh_token = std::env::var("USER_REFRESH_TOKEN")
        .expect("No USER_REFRESH_TOKEN in env var")
        .as_str().to_string();

    // Construct UserSecret
    let client_token = UserSecret {
        client_id: client_id,
        client_secret: client_secret,
        refresh_token: refresh_token,
    };

    // Auth to Token, will panic if failed.
    let _token = client_token.auth().await
        .expect("Unable to authenticate");
}

#[tokio::test]
async fn test_auth_service() {
    let client_email = std::env::var("SERVICE_CLIENT_EMAIL")
        .expect("No SERVICE_CLIENT_EMAIL in env var")
        .as_str().to_string();
    let private_key = std::env::var("SERVICE_PRIVATE_KEY")
        .expect("No SERVICE_PRIVATE_KEY in env var")
        .as_str().to_string();
    let private_key_id = std::env::var("SERVICE_PRIVATE_KEY_ID")
        .expect("No SERVICE_PRIVATE_KEY_ID in env var")
        .as_str().to_string();

    let service_secret = ServiceSecret {
        client_email: client_email,
        private_key: private_key,
        private_key_id: private_key_id,
    };

    let scopes: Vec<String> = vec![
        "https://www.googleapis.com/auth/drive".to_string(),
        "https://www.googleapis.com/auth/youtube".to_string()
    ];

    let scope = scopes.join(" ");

    let _token = service_secret.auth(&scope).await
        .expect("Unable to authenticate");
}


