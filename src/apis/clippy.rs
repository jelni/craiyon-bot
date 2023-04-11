use reqwest::header::ORIGIN;
use reqwest::Url;
use serde::Serialize;

use crate::commands::CommandError;
use crate::utilities::api_utils::DetectServerError;

#[derive(Serialize)]
struct Payload<'a> {
    query: &'a str,
}

pub async fn query(http_client: reqwest::Client, query: &str) -> Result<String, CommandError> {
    let response = http_client
        .post(Url::parse("https://api.clippy.help/widget/stream").unwrap())
        .json(&Payload { query })
        .header(ORIGIN, "https://docs.hop.io/")
        .send()
        .await?
        .server_error()?
        .error_for_status()?
        .text()
        .await?;

    let reply = parse_response(&response);

    Ok(reply)
}

fn parse_response(response: &str) -> String {
    let mut lines = response.lines();
    let mut response = String::new();

    while let Some(line) = lines.next() {
        if line != "id:partial_answer" {
            continue;
        }

        if let Some(line) = lines.next().and_then(|line| line.strip_prefix("data:")) {
            response.push_str(line.strip_prefix(' ').unwrap_or(line));
        }
    }

    response
}
