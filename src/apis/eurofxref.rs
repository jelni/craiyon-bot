use serde::Deserialize;

#[derive(Deserialize)]
struct Cube<T> {
    #[serde(rename = "Cube")]
    cube: T,
}

#[derive(Deserialize)]
pub struct Rate {
    #[serde(rename = "@currency")]
    pub currency: String,
    #[serde(rename = "@rate")]
    pub rate: String,
}

pub async fn daily(client: &reqwest::Client) -> reqwest::Result<Vec<Rate>> {
    let response =
        client.get("https://www.ecb.europa.eu/stats/eurofxref/eurofxref-daily.xml").send().await?;

    let result =
        serde_xml_rs::from_str::<Cube<Cube<Cube<Vec<Rate>>>>>(&response.text().await.unwrap())
            .unwrap();

    Ok(result.cube.cube.cube)
}
