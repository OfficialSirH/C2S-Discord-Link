use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Deserialize, PostgresMapper, Serialize)]
#[allow(non_snake_case)]
#[pg_mapper(table = "UserData")]
pub struct UserData {
    pub discordId: String,
    pub token: String,
    pub betaTester: bool,
    pub metabits: i64,
    pub dino_rank: i32,
    pub prestige_rank: i32,
    pub singularity_speedrun_time: f32,
    pub all_sharks_obtained: bool,
    pub all_hidden_achievements_obtained: bool,
    pub edited_timestamp: SystemTime,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct ReceivedUserData {
    pub betaTester: bool,
    pub metabits: i64,
    pub dino_rank: i32,
    pub prestige_rank: i32,
    pub singularity_speedrun_time: f32,
    pub all_sharks_obtained: bool,
    pub all_hidden_achievements_obtained: bool,
    pub edited_timestamp: SystemTime,
}
