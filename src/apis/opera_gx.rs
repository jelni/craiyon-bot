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
    let token = http_client
        .post("https://api.discord.gx.games/v1/direct-fulfillment")
        .json(&Payload {
            partner_user_id: "510429d266a6a5e2374f80a2942c7cfe7fc317b21b7b31b06d4a54a0287aacb8",
        })
        .header(ACCEPT, "application/json")
        .send()
        .await?
        .server_error()?
        .json::<Response>()
        .await?
        .token;

    Ok(format!("https://discord.com/billing/partner-promotions/1180231712274387115/{token}"))
}
