
use std::env;
mod auth_users;
use auth_users::{
    UserSecret,
    ClientSecret
};

mod auth_service;
use auth_service::ServiceSecret;

use serde_json;

mod token;


const HELP_MSG: &str = "
Usage: gapi-oauth <SERVICE> <JSON_PATH> [SCOPE] [PORT]

SERVICE: `user`, `service`, or `consent`
JSON_PATH: The path to the JSON file containing the credentials.
SCOPE: Only required for `service` and `consent`
PORT: Only required for `consent`
";

async fn user_service(fpath: &str) -> Result<String, std::io::Error> {
    let user_secret = UserSecret::from_file(fpath)?;
    let token = user_secret.auth().await
        .expect("Failed to authenticate user");
    let json_str = serde_json::to_string_pretty(&token)
        .expect("Failed to convert token to JSON string");
    Ok(json_str)
}

async fn service_service(fpath: &str, scope: &str) -> Result<String, std::io::Error> {
    let service_secret = ServiceSecret::from_file(fpath)?;
    let token = service_secret.auth(scope).await
        .expect("Failed to authenticate user");
    let json_str = serde_json::to_string_pretty(&token)
        .expect("Failed to convert token to JSON string");
    Ok(json_str)
}

async fn consent_service(fpath: &str, scope: &str, port: u32) -> Result<String, std::io::Error> {
    let client_secret = ClientSecret::from_file(&fpath)?;
    let token = client_secret.auth(scope, port).await
        .expect("Failed to authorize");
    let json_str = serde_json::to_string_pretty(&token)
        .expect("Failed to convert token to JSON string");
    Ok(json_str)
}


#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 5 || args.len() == 1 {
        println!("{}", HELP_MSG);
        panic!("");
    }

    match args[1].as_str() {
        "user" => {
            if args.len() != 3 {
                println!("{}", HELP_MSG);
                panic!("");
            }
            let token_str = user_service(&args[2])
                .await
                .expect("Failed authenticate user");
            println!("{}", token_str);
        },
        "service" => {
            if args.len() != 4 {
                println!("{}", HELP_MSG);
                panic!("");
            }
            let token_str = service_service(&args[2], &args[3])
                .await
                .expect("Failed authenticate user");
            println!("{}", token_str);
        },
        "consent" => {
            if args.len()!= 5 {
                println!("{}", HELP_MSG);
                panic!("");
            }
            let token_str = consent_service(&args[2], &args[3], args[4].parse::<u32>().unwrap())
                .await
                .expect("Failed authenticate user");
            println!("{}", token_str);
        },
        _ => {
            println!("Only `user` or `service` is allowed.");
            panic!("");
        }
    };
}