use core::fmt;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use md5::Digest;
use reqwest::header::{CONTENT_TYPE, ORIGIN};
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
    image: &[u8],
) -> reqwest::Result<Result<Media, ProcessingError>> {
    let json = serde_json::ser::to_string(&InputData {
        busi_id: "different_dimension_me_img_entry",
        images: vec![STANDARD.encode(image)],
    })
    .unwrap();

    let signature = format!("{:x}", sign(&json));

    loop {
        let result = http_client
            .post("https://ai.tu.qq.com/trpc.shadow_cv.ai_processor_cgi.AIProcessorCgi/Process")
            .body(json.clone())
            .header(CONTENT_TYPE, "application/json")
            .header(ORIGIN, "https://h5.tu.qq.com")
            .header("x-sign-value", &signature)
            .header("x-sign-version", "v1")
            .send()
            .await?
            .json::<ProcessingResult>()
            .await?;

        break Ok(if let Some(extra) = result.extra {
            Ok(serde_json::from_str::<Media>(&extra).unwrap())
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

fn sign(data: &str) -> Digest {
    md5::compute(format!("https://h5.tu.qq.com{}HQ31X02e", data.len()))
}
