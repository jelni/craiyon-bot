use rand::distributions::{Alphanumeric, DistString};
use rand::rngs::StdRng;
use rand::{CryptoRng, SeedableRng};
use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize};

use crate::commands::CommandError;
use crate::utilities::api_utils::DetectServerError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Payload<'a> {
    partner_user_id: &'a str,
}

#[derive(Deserialize)]
struct Response {
    token: String,
}

pub async fn generate(http_client: reqwest::Client) -> Result<String, CommandError> {
    let partner_user_id = &Alphanumeric.sample_string(&mut StdRng::from_entropy(), 64);

    let token = http_client
        .post("https://api.discord.gx.games/v1/direct-fulfillment")
        .json(&Payload { partner_user_id })
        .header(ACCEPT, "application/json")
        .send()
        .await?
        .server_error()?
        .json::<Response>()
        .await?
        .token;

    Ok(format!("https://discord.com/billing/partner-promotions/1180231712274387115/{token}"))
}
