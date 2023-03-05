use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: String,
    display_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GraphResponse<T> {
    value: Vec<T>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // read environment variables
    // NOTE: ChatGPT used "?" instead of "expect" to handle errors
    let client_id = env::var("AZURE_CLIENT_ID").expect("AZURE_CLIENT_ID must be set");
    let client_secret = env::var("AZURE_CLIENT_SECRET").expect("AZURE_CLIENT_SECRET must be set");
    let tenant_id = env::var("AZURE_TENANT_ID").expect("AZURE_TENANT_ID must be set");

    // Build authentication token URL
    let auth_url = format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
        tenant_id
    );

    // Build token request body
    // NOTE: here ChatGPT suggested to use the graph URL without escaped characters
    let token_request_body = format!(
        "client_id={}&scope=https%3A%2F%2Fgraph.microsoft.com%2F.default&client_secret={}&grant_type=client_credentials",
        client_id, client_secret
    );

    // Send token request and parse response
    let client = reqwest::Client::new();
    let token_response = client
        .post(&auth_url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(token_request_body)
        .send()
        .await?
        .json::<Value>()
        .await?;

    let access_token = token_response["access_token"].as_str().unwrap();

    // Build graph request URL
    // NOTE: from here Co-Pilot copied the comments from ChatGPT
    let graph_url = "https://graph.microsoft.com/v1.0/users";
    let graph_request_url = format!("{}?select=id,displayName", graph_url);

    // Send Graph API request and parse response
    let graph_response = client
        .get(&graph_request_url)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?
        .json::<GraphResponse<User>>()
        .await?;

    for user in graph_response.value {
        println!("ID: {}, Display Name: {}", user.id, user.display_name);
    }

    Ok(())
}