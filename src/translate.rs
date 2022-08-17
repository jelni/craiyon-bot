use reqwest::Url;
use serde::Deserialize;

#[derive(Deserialize)]
struct Response {
    pub sentences: Vec<Sentence>,
    pub src: String,
}

#[derive(Deserialize)]
struct Sentence {
    pub trans: String,
}

pub struct Translation {
    pub text: String,
    pub source_language: String,
}

pub async fn translate<S: AsRef<str>>(
    http_client: reqwest::Client,
    text: S,
    source_language: Option<&str>,
    translation_language: &str,
) -> reqwest::Result<Translation> {
    let response = http_client
        .get(
            Url::parse_with_params(
                "https://translate.googleapis.com/translate_a/single",
                [
                    ("client", "gtx"),
                    ("sl", source_language.unwrap_or("auto")),
                    ("tl", translation_language),
                    ("dt", "t"),
                    ("dj", "1"),
                    ("q", text.as_ref()),
                ],
            )
            .unwrap(),
        )
        .send()
        .await?
        .error_for_status()?
        .json::<Response>()
        .await?;

    Ok(Translation {
        text: response.sentences.into_iter().map(|s| s.trans).collect(),
        source_language: response.src,
    })
}
