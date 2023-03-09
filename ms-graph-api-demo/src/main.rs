use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;

use std::time::Duration;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};

use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: String,
    #[serde(rename = "displayName")]
    display_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GraphResponse<T> {
    value: Vec<T>,
}

// struct for error
#[derive(Debug, Serialize, Deserialize)]
struct GraphError {
    error: GraphErrorDetail,
}

#[derive(Debug, Serialize, Deserialize)]
struct GraphErrorDetail {
    code: String,
    message: String,
    #[serde(rename = "innerError")]
    inner_error: GraphErrorDetailInner,
}

#[derive(Debug, Serialize, Deserialize)]
struct GraphErrorDetailInner {
    date: String,
    #[serde(rename = "request-id")]
    request_id: String,
}

#[tokio::main]
async fn _main() -> Result<(), Box<dyn std::error::Error>> {
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
    // NOTE: ChatGPT made a mistake here by using select instead of $select
    let graph_url = "https://graph.microsoft.com/v1.0/users";
    let graph_request_url = format!("{}?$select=id,displayName", graph_url);

    // Send Graph API request and parse response
    // write raw response to variable
    let graph_response = client
        .get(&graph_request_url)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?;
    
    match graph_response.status() {
        reqwest::StatusCode::OK => {
            let graph_response = graph_response.json::<GraphResponse<User>>().await?;
            for user in graph_response.value {
                println!("ID: {}, Display Name: {}", user.id, user.display_name);
            }
        }
        _ => {
            let error = graph_response.json::<GraphError>().await?.error;
            match error.code.as_str() {
                "Authorization_RequestDenied" => {
                    println!("Authorization Faild: {}", error.message);
                }
                _ => {
                    println!("Error: {}", error.message);
                }
            }
        }
    }

    Ok(())
}

// Define the data types for the request and response payloads
#[derive(Debug, Serialize)]
struct DeviceCodeRequest<'a> {
    client_id: &'a str,
    scope: &'a str,
}

#[derive(Debug, Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u32,
    interval: u32,
}

#[derive(Debug, Deserialize)]
struct AccessTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u32,
    refresh_token: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client_id = env::var("AZURE_CLIENT_ID").expect("AZURE_CLIENT_ID must be set");
    let tenant_id = env::var("AZURE_TENANT_ID").expect("AZURE_TENANT_ID must be set");

    // Set up the HTTP client and request payload
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let device_code_request = DeviceCodeRequest {
        client_id: client_id.as_str(),
        scope: "Mail.Read",
    };

    // Send the device code request and parse the response
    let response = client.post(format!("https://login.microsoftonline.com/{}/oauth2/v2.0/devicecode", tenant_id))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .form(&device_code_request)
        .send()
        .await?;

    // store json and print it, and afterwards convert to struct
    let json = response.json::<Value>().await?;
    println!("{:?}", json);
    //let device_code_response = json::<DeviceCodeResponse>().await?;
    // in variable json is the json response from the server
    // take this text and parse it to json and put it into the DeviceCodeResponse struct
    let device_code_response = serde_json::from_str::<DeviceCodeResponse>(&json.to_string())?;
     
    // Display the user code and verification URI for the user to authenticate
    println!("Please authenticate using the following code: {}", device_code_response.user_code);
    println!("Visit this URL to authenticate: {}", device_code_response.verification_uri);

    // Poll for the access token until the authorization server has authenticated the user
    let mut interval = Duration::from_secs(u64::from(device_code_response.interval));
    println!("{:?}", interval);
    loop {
        // Wait for the specified interval before polling again
        std::thread::sleep(interval);

        // Send the access token request and parse the response
        let response = client.post(format!("https://login.microsoftonline.com/{}/oauth2/v2.0/token", tenant_id))
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .form(&[
                ("client_id", client_id.as_str()),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("device_code", &device_code_response.device_code),
            ])
            .send()
            .await?;
    
        let access_token_response: serde_json::Value = response.json().await?;    
        // print the raw response
        println!("{:?}", access_token_response);
        if let Some(access_token) = access_token_response["access_token"].as_str() {
            println!("Access token: {}", access_token);
            break;
        }

        // If the access token has not yet been granted, update the interval and continue polling
        // NOTE: ChatGPT set this one here for the expire_in value -> this would mean to wait until the token is expired
        interval = Duration::from_secs(5);
        // print something to know that the loop is still running
        println!("waiting for token");
    }

    Ok(())
}