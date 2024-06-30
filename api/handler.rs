use expo_push_notification_client::{Expo, ExpoClientOptions, ExpoPushMessage};
use serde_json::json;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    println!("this is a expo push notification api");

    // Initialize Expo client
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

    // exstract title and body from the request
    let body = req.body();
    println!("Request body1: {:?}", body);
    println!("Request body2: {:?}", body);

    // Expo push tokens (example token used here)
    // get this token from the body of the request
    let expo_push_tokens = ["ExponentPushToken[xxxxxxxxxxxxxxxxxxxxxx]"];

    // Build the push message
    let expo_push_message = ExpoPushMessage::builder(expo_push_tokens)
        .title("Test ")
        .body("This is a test notification")
        .build()
        .map_err(|e| Error::from(e))?;

    // Send push notifications
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
