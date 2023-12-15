use serde::{Deserialize, Serialize};

use crate::commands::CommandError;

const QUERY: &str = "query systemStats($systemId: ID!) {
  systemActiveTripCount(systemId: $systemId) {
    count
  }
  systemStats(systemId: $systemId) {
    tripsToday
    tripsYesterday
    uniqueUsersToday
    medianDurationToday
    medianDurationThisYear
  }
}";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Request {
    operation_name: &'static str,
    variables: Variables,
    query: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Variables {
    system_id: &'static str,
}

#[derive(Deserialize)]
pub struct Response {
    data: Data,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub system_active_trip_count: SystemActiveTripCount,
    pub system_stats: SystemStats,
}

#[derive(Deserialize)]
pub struct SystemActiveTripCount {
    pub count: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStats {
    pub trips_today: u32,
    pub trips_yesterday: u32,
    pub unique_users_today: u32,
    pub median_duration_today: u32,
    pub median_duration_this_year: u32,
}

pub async fn system_stats(
    http_client: reqwest::Client,
    system_id: &'static str,
) -> Result<Data, CommandError> {
    let response = http_client
        .post("https://core.urbansharing.com/public/api/v1/graphql")
        .json(&Request {
            operation_name: "systemStats",
            variables: Variables { system_id },
            query: QUERY,
        })
        .send()
        .await?;

    Ok(response.json::<Response>().await?.data)
}
