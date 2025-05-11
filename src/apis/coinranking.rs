use serde::Deserialize;

#[derive(Deserialize)]
struct Response {
    data: Data,
}

#[derive(Deserialize)]
struct Data {
    coins: Vec<Coin>,
}

#[derive(Deserialize)]
pub struct Coin {
    pub symbol: String,
    pub price: String,
}

pub async fn coins(client: &reqwest::Client) -> reqwest::Result<Vec<Coin>> {
    let response = client.get("https://api.coinranking.com/v2/coins").send().await?;
    let result = response.json::<Response>().await?;

    Ok(result.data.coins)
}
