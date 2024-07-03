use expo_push_notification_client::{Expo, ExpoClientOptions, ExpoPushMessage};
use serde_json::{json, Value};
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    println!("this is a expo push notification api");

    let expo = Expo::new(ExpoClientOptions {
        ..Default::default()
    });

    if req.method() != "POST" {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(
                json!({
                    "error": "Only POST requests are allowed"
                })
                .to_string()
                .into(),
            )?);
    }

    let mut expo_push_tokens = vec![];

    let body = req.body();
    let body_str = String::from_utf8(body.to_vec()).map_err(|e| Error::from(e))?;
    let json_body: Value = serde_json::from_str(&body_str).map_err(|e| Error::from(e))?;

    let title = json_body["title"].as_str().map_err(|e| Error::from(e))?;
    let body = json_body["body"].as_str().map_err(|e| Error::from(e))?;
    expo_push_tokens.push(json_body["expo_push_token"].as_str()
        .map_err(|e| Error::from(e))?;

    let expo_push_message = ExpoPushMessage::builder(expo_push_tokens)
        .title(title)
        .body(body)
        .build()
        .map_err(|e| Error::from(e))?;

    match expo.send_push_notifications(expo_push_message).await {
        Ok(_ret) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(
                json!({
                    "message": "Push notification sent successfully"
                })
                .to_string()
                .into(),
            )?),
        Err(_e) => Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("Content-Type", "application/json")
            .body(
                json!({
                    "error": "Failed to send push notification"
                })
                .to_string()
                .into(),
            )?),
    }
}
