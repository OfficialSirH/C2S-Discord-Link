use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "UserData")]
pub struct UserData {
    pub discord_id: String,
    pub token: String,
    pub beta_tester: bool,
    pub metabits: i64,
    pub dino_rank: i32,
    pub prestige_rank: i32,
    pub beyond_rank: i32,
    pub singularity_speedrun_time: Option<f64>,
    pub all_sharks_obtained: bool,
    pub all_hidden_achievements_obtained: bool,
    pub edited_timestamp: SystemTime,
}

#[derive(Deserialize)]
pub struct OGUpdateUserData {
    #[serde(rename = "playerToken")]
    pub player_token: String,
    #[serde(rename = "betaTester")]
    pub beta_tester: bool,
    pub metabits: f64,
    pub dino_rank: i32,
    pub prestige_rank: i32,
    pub beyond_rank: i32,
    pub singularity_speedrun_time: Option<f64>,
    pub all_sharks_obtained: bool,
    pub all_hidden_achievements_obtained: bool,
}

#[derive(Deserialize)]
pub struct UpdateUserData {
    pub beta_tester: bool,
    pub metabits: f64,
    pub dino_rank: i32,
    pub prestige_rank: i32,
    pub beyond_rank: i32,
    pub singularity_speedrun_time: Option<f64>,
    pub all_sharks_obtained: bool,
    pub all_hidden_achievements_obtained: bool,
}

impl From<OGUpdateUserData> for UpdateUserData {
    fn from(data: OGUpdateUserData) -> Self {
        UpdateUserData {
            beta_tester: data.beta_tester,
            metabits: data.metabits,
            dino_rank: data.dino_rank,
            prestige_rank: data.prestige_rank,
            beyond_rank: data.beyond_rank,
            singularity_speedrun_time: data.singularity_speedrun_time,
            all_sharks_obtained: data.all_sharks_obtained,
            all_hidden_achievements_obtained: data.all_hidden_achievements_obtained,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateUserData {
    pub discord_id: String,
    pub beta_tester: bool,
}

#[derive(Deserialize)]
pub struct DeleteUserData {
    pub discord_id: String,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub message: String,
}

/// response structure for game saves metadata
#[derive(Deserialize)]
pub struct GameSavesMetadataResponse {
    #[serde(rename = "responseType")]
    pub response_type: String,
    pub url: String,
    pub error: Option<String>,
    #[serde(rename = "fileSize")]
    pub file_size: Option<i64>,
    #[serde(rename = "dateUpdated")]
    pub date_updated: Option<i64>,
    #[serde(rename = "playTime")]
    pub play_time: Option<i64>,
}

/// request structure for retrieving game saves metadata
#[derive(Deserialize)]
pub struct GameSavesMetadataRequest {
    /// should *always* be "getmetadata"
    pub action: String,
    /// user email
    pub username: String,
    /// user access token
    pub token: String,
}
