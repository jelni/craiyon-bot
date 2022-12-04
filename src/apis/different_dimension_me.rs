use core::fmt;

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InputData {
    busi_id: &'static str,
    images: Vec<String>,
}

#[derive(Deserialize)]
struct ProcessingResult {
    code: i32,
    msg: String,
    extra: Option<String>,
}

#[derive(Deserialize)]
pub struct Media {
    pub img_urls: Vec<String>,
}

pub struct ProcessingError {
    pub code: i32,
    pub message: String,
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Different Dimension error {}: {}", self.code, self.message)
    }
}

pub async fn process(
    http_client: reqwest::Client,
    image: Vec<u8>,
) -> reqwest::Result<Result<Media, ProcessingError>> {
    loop {
        let result = http_client
            .post("https://ai.tu.qq.com/trpc.shadow_cv.ai_processor_cgi.AIProcessorCgi/Process")
            .json(&InputData {
                busi_id: "ai_painting_anime_img_entry",
                images: vec![base64::encode(&image)],
            })
            .send()
            .await?
            .json::<ProcessingResult>()
            .await?;

        break Ok(if let Some(extra) = result.extra {
            Ok(serde_json::de::from_str::<Media>(&extra).unwrap())
        } else {
            let err = ProcessingError { code: result.code, message: result.msg };
            if err.message == "VOLUMN_LIMIT" {
                log::warn!("{err}");
                continue;
            }
            Err(err)
        });
    }
}
