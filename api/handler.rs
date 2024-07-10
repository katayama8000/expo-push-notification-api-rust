use expo_push_notification_client::{Expo, ExpoClientOptions, ExpoPushMessage};
use serde_json::{json, Value};
use supabase_rs::SupabaseClient;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

use dotenv::dotenv;
use std::env::var;
use tracing::{error, info};

async fn initialize_supabase_client() -> Result<SupabaseClient, Error> {
    dotenv().ok();

    let supabase_url = var("SUPABASE_URL").map_err(|e| {
        eprintln!("Error loading SUPABASE_URL: {:?}", e);
        Error::from(e)
    })?;
    let supabase_key = var("SUPABASE_KEY").map_err(|e| {
        eprintln!("Error loading SUPABASE_KEY: {:?}", e);
        Error::from(e)
    })?;

    Ok(SupabaseClient::new(supabase_url, supabase_key))
}

async fn fetch_expo_push_tokens(client: &SupabaseClient) -> Result<Vec<String>, Error> {
    let response = client.select("dev_users").execute().await.map_err(|e| {
        eprintln!("Error fetching dev_users: {:?}", e);
        Error::from(e)
    })?;

    let tokens = response
        .iter()
        .filter_map(|row| row["expo_push_token"].as_str().map(|s| s.to_string()))
        .collect::<Vec<String>>();
    info!("Fetched {} expo push tokens", tokens.len());
    Ok(tokens)
}

async fn extract_body(req: &Request) -> Result<Value, Error> {
    let body_str = String::from_utf8(req.body().to_vec()).map_err(|e| {
        eprintln!("Error converting body to string: {:?}", e);
        Error::from(e)
    })?;
    let json_body: Value = serde_json::from_str(&body_str).map_err(|e| {
        eprintln!("Error parsing JSON body: {:?}", e);
        Error::from(e)
    })?;
    Ok(json_body)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    println!("This is an Expo push notification API");

    let expo = Expo::new(ExpoClientOptions::default());

    let mut title = "25日だよ".to_string();
    let mut body = "パートナーに請求しよう".to_string();
    let mut expo_push_tokens = vec![];

    if req.method() == "GET" {
        let supabase_client = initialize_supabase_client().await?;
        expo_push_tokens = fetch_expo_push_tokens(&supabase_client).await?;
    }

    if req.method() == "POST" {
        let json_body = extract_body(&req).await?;
        title = json_body["title"].to_string();
        body = json_body["body"].to_string();
        info!("Title: {}", title);
        info!("Body: {}", body);
        info!("expo_push_token: {:?}", json_body["expo_push_token"]);

        if let Some(token) = json_body["expo_push_token"].as_str() {
            expo_push_tokens.push(token.to_string());
        }
    }

    let expo_push_message = ExpoPushMessage::builder(expo_push_tokens)
        .title(title)
        .body(body)
        .build()
        .map_err(Error::from)?;

    match expo.send_push_notifications(expo_push_message).await {
        Ok(_) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(
                json!({
                    "message": "Push notification sent successfully"
                })
                .to_string()
                .into(),
            )?),
        Err(e) => {
            error!("Failed to send push notification, {:?}", e);
            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "application/json")
                .body(
                    json!({
                        "error": "Failed to send push notification"
                    })
                    .to_string()
                    .into(),
                )?)
        }
    }
}
