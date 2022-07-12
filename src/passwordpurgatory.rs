use reqwest::Url;
use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct Response {
    pub message: String,
}

pub async fn make_hell<S: Into<String>>(
    http_client: reqwest::Client,
    password: S,
) -> reqwest::Result<String> {
    let message = http_client
        .post(
            Url::parse_with_params(
                "https://api.passwordpurgatory.com/make-hell",
                [("password", password.into())],
            )
            .unwrap(),
        )
        .send()
        .await?
        .error_for_status()?
        .json::<Response>()
        .await?
        .message;

    Ok(message)
}
