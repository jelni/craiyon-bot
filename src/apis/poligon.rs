use serde::Deserialize;

#[derive(Deserialize)]
struct Response {
    joke: String,
}

pub async fn startit_joke(http_client: reqwest::Client) -> reqwest::Result<String> {
    let response = http_client
        .get("https://astolfo.poligon.lgbt/startit")
        .send()
        .await?
        .json::<Response>()
        .await?;

    Ok(response.joke)
}
