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
    pub singularity_speedrun_time: Option<f64>,
    pub all_sharks_obtained: bool,
    pub all_hidden_achievements_obtained: bool,
    pub edited_timestamp: SystemTime,
}

#[derive(Deserialize)]
pub struct ReceivedUserData {
    #[serde(rename = "playerToken")]
    pub player_token: String,
    #[serde(rename = "betaTester")]
    pub beta_tester: bool,
    pub metabits: f64,
    pub dino_rank: i32,
    pub prestige_rank: i32,
    pub singularity_speedrun_time: Option<f64>,
    pub all_sharks_obtained: bool,
    pub all_hidden_achievements_obtained: bool,
}

#[derive(Deserialize)]
pub struct DataTypeAccurateUserData {
    pub player_token: String,
    pub beta_tester: bool,
    pub metabits: i64,
    pub dino_rank: i32,
    pub prestige_rank: i32,
    pub singularity_speedrun_time: Option<f64>,
    pub all_sharks_obtained: bool,
    pub all_hidden_achievements_obtained: bool,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub message: String,
}
