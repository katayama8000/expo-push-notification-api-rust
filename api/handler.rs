use expo_push_notification_client::{Expo, ExpoClientOptions, ExpoPushMessage};
use serde_json::{json, Value};
use supabase_rs::SupabaseClient;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

use dotenv::dotenv;
use std::env::var;

enum SupabaseError {
    SupabaseError,
}

impl From<SupabaseError> for Error {
    fn from(_: SupabaseError) -> Self {
        Error::from("SupabaseError")
    }
}

async fn initialize_supabase_client() -> Result<SupabaseClient, SupabaseError> {
    dotenv().ok();

    let supabase_url = var("SUPABASE_URL").map_err(|e| {
        eprintln!("Error loading SUPABASE_URL: {:?}", e);
        SupabaseError::SupabaseError
    })?;
    let supabase_key = var("SUPABASE_KEY").map_err(|e| {
        eprintln!("Error loading SUPABASE_KEY: {:?}", e);
        SupabaseError::SupabaseError
    })?;

    Ok(
        SupabaseClient::new(supabase_url, supabase_key).map_err(|e| {
            eprintln!("Error initializing SupabaseClient: {:?}", e);
            SupabaseError::SupabaseError
        })?,
    )
}

async fn fetch_expo_push_tokens(client: &SupabaseClient) -> Result<Vec<String>, Error> {
    let response = client.select("users").execute().await.map_err(|e| {
        eprintln!("Error fetching dev_users: {:?}", e);
        Error::from(e)
    })?;

    let tokens = response
        .iter()
        .filter_map(|row| row["expo_push_token"].as_str().map(|s| s.to_string()))
        .collect::<Vec<String>>();
    println!("fetched expo push tokens from supabase {:?}", tokens);
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
    println!(
        "This is an Expo push notification API ver: {}",
        env!("CARGO_PKG_VERSION"),
    );
    println!("Request Headers: {:?}", req.headers());

    let expo = Expo::new(ExpoClientOptions::default());

    let mut title = "25日だよ".to_string();
    let mut body = "パートナーに請求しよう".to_string();
    let mut expo_push_tokens = vec![];

    match req.method().as_str() {
        "GET" => {
            let request_secret = req
                .headers()
                .get("X-Vercel-Cron-Secret")
                .and_then(|value| value.to_str().ok())
                .unwrap_or("");

            println!("Request Secret: {}", request_secret);

            let supabase_client = initialize_supabase_client().await?;
            expo_push_tokens = fetch_expo_push_tokens(&supabase_client).await?;
        }
        "POST" => {
            let json_body = extract_body(&req).await?;

            if let Some(t) = json_body["title"].as_str() {
                title = t.to_string();
            } else {
                eprintln!("Title is required");
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header("Content-Type", "application/json")
                    .body(
                        json!({
                            "error": "Title is required"
                        })
                        .to_string()
                        .into(),
                    )?);
            }

            if let Some(b) = json_body["body"].as_str() {
                body = b.to_string();
            } else {
                eprintln!("Body is required");
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header("Content-Type", "application/json")
                    .body(
                        json!({
                            "error": "Body is required"
                        })
                        .to_string()
                        .into(),
                    )?);
            }

            if let Some(token) = json_body["expo_push_token"].as_str() {
                if Expo::is_expo_push_token(token) {
                    expo_push_tokens.push(token.to_string());
                } else {
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .header("Content-Type", "application/json")
                        .body(
                            json!({
                                "error": "Invalid expo push token"
                            })
                            .to_string()
                            .into(),
                        )?);
                }
            }
            println!("Title: {}", title);
            println!("Body: {}", body);
            println!("expo_push_tokens: {:?}", expo_push_tokens);
        }
        _ => {
            return Ok(Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .header("Content-Type", "application/json")
                .body(
                    json!({
                        "error": "Method not allowed"
                    })
                    .to_string()
                    .into(),
                )?);
        }
    }

    println!("Building push notification");
    let expo_push_message = ExpoPushMessage::builder(expo_push_tokens)
        .title(title)
        .body(body)
        .build()
        .map_err(|e| {
            eprintln!("Error building ExpoPushMessage: {:?}", e);
            Error::from(e)
        })?;

    println!("Sending push notification");
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
            eprintln!("Failed to send push notification: {:?}", e);
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
