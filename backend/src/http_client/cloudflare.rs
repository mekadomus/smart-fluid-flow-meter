use serde::{Deserialize, Serialize};

use tracing::error;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CheckCaptchaRequest<'a> {
    pub secret: &'a str,
    pub response: &'a str,
}

#[derive(Debug, serde::Deserialize)]
struct CheckCaptchaResponse {
    success: bool,
}

pub async fn check_captcha(
    request: CheckCaptchaRequest<'_>,
) -> Result<bool, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let resp = client
        .post("https://challenges.cloudflare.com/turnstile/v0/siteverify")
        .header("accept", "application/json")
        .header("content-type", "application/json")
        .json(&request)
        .send()
        .await;

    match resp {
        Ok(res) => match res.json::<CheckCaptchaResponse>().await {
            Ok(j) => {
                return Ok(j.success);
            }
            Err(err) => {
                error!("Err 2: {:?}", err);
                return Err(Box::new(err));
            }
        },
        Err(err) => {
            error!("Err 1: {:?}", err);
            return Err(Box::new(err));
        }
    }
}
