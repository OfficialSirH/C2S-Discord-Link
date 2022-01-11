mod config {
    use dotenv::vars;

    pub struct Config {
        pub discord_token: String,
        pub userdata_auth: String,
        pub server_addr: String,
        pub pg: deadpool_postgres::Config,
    }
    impl Config {
        pub fn new() -> Self {
            let environment_vars: Vec<(String, String)> = vars().collect();
            Config {
                discord_token: find_key(&environment_vars, "DISCORD_TOKEN"),
                userdata_auth: find_key(&environment_vars, "USERDATA_AUTH"),
                server_addr: find_key(&environment_vars, "SERVER_ADDR"),
                pg: deadpool_postgres::Config::new(),
            }
        }
    }

    pub fn find_key(iteration: &Vec<(String, String)>, key_search: &'static str) -> String {
        match iteration.iter().find(|(key, _)| key == key_search) {
            Some((_, value)) => value.to_string(),
            None => panic!("couldn't find '{}' in the environment variables", key_search),
        }
    }
}

mod models {
    use std::time::SystemTime;
    use serde::{Deserialize, Serialize};
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
        pub edited_timestamp: SystemTime
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
        pub edited_timestamp: SystemTime
    }
}

mod errors {
    use actix_web::{HttpResponse, ResponseError};
    use deadpool_postgres::PoolError;
    use derive_more::{Display, From};
    use tokio_pg_mapper::Error as PGMError;
    use tokio_postgres::error::Error as PGError;

    #[derive(Display, From, Debug)]
    pub enum MyError {
        NotFound,
        PGError(PGError),
        PGMError(PGMError),
        PoolError(PoolError),
    }
    impl std::error::Error for MyError {}

    impl ResponseError for MyError {
        fn error_response(&self) -> HttpResponse {
            match *self {
                MyError::NotFound => HttpResponse::NotFound().finish(),
                MyError::PoolError(ref err) => {
                    HttpResponse::InternalServerError().body(err.to_string())
                }
                _ => HttpResponse::InternalServerError().finish(),
            }
        }
    }
}

mod db {
    use crate::models::{UserData, ReceivedUserData};
    use deadpool_postgres::Client;
    use tokio_pg_mapper::{FromTokioPostgresRow,Error};

    pub async fn get_userdata(client: &Client, token: &str) -> Result<UserData, Error> {
        let _stmt = include_str!("../sql/get_userdata.sql");
        let _stmt = _stmt.replace("$token", &token);
        let stmt = client.prepare(&_stmt).await?;

        let queried_data = client.query(&stmt, &[]).await?.pop().ok_or(Error::ColumnNotFound);

        UserData::from_row_ref(&queried_data?)
    }

    pub async fn update_userdata(client: &Client, token: &str, user_data: ReceivedUserData) -> Result<UserData, Error> {
        let _stmt = include_str!("../sql/update_userdata.sql");
        let _stmt = _stmt.replace("$token", &token);
        let stmt = client.prepare(&_stmt).await?;

        let queried_data = client
            .query(
                &stmt,
                &[
                    &user_data.betaTester,
                    &user_data.metabits,
                    &user_data.dino_rank,
                    &user_data.prestige_rank,
                    &user_data.singularity_speedrun_time,
                    &user_data.all_sharks_obtained,
                    &user_data.all_hidden_achievements_obtained,
                    &std::time::SystemTime::now()
                ]
            )
            .await?
            .pop().ok_or(Error::ColumnNotFound);

        UserData::from_row_ref(&queried_data?)
    }
}

mod handlers {
    use std::str::from_utf8;
    use crate::{db, errors::MyError, models::ReceivedUserData};
    use actix_web::{web, Error, HttpResponse};
    use deadpool_postgres::{Client, Pool};
    use hmac::{Hmac, Mac};
    use serde::{Serialize, Deserialize};

    #[derive(Serialize)]
    struct MessageResponse {
    message: String
    }

    type HmacSha256 = Hmac<sha2::Sha256>;

    #[allow(non_snake_case)]
    #[derive(Deserialize)]
    pub struct PlayerData {
        playerId: String,
        playerToken: String
    }

    pub async fn update_user(
        query: web::Query<PlayerData>,
        received_user: web::Json<ReceivedUserData>,
        db_pool: web::Data<Pool>,
    ) -> Result<HttpResponse, Error> {
        let user_data: ReceivedUserData = received_user.into_inner();

        let client: Client = db_pool.get().await.map_err(MyError::PoolError)?;
        let config = crate::config::Config::new();

        let mut mac = match HmacSha256::new_from_slice(&config.userdata_auth.as_bytes()) {
            Ok(hmac_data) => hmac_data,
            Err(_) => return Ok(HttpResponse::Ok().json(MessageResponse {
                message: "request failed at token-creation process, please try again".to_string()
            }))
        };
        mac.update(query.playerId.as_bytes());
        mac.update(query.playerToken.as_bytes());
        
        let user_token = mac.finalize().into_bytes();
        let user_token = match from_utf8(&user_token) {
            Ok(token) => token,
            Err(_) => return Ok(HttpResponse::Ok().json(MessageResponse {
                message: "request failed at token-creation process, please try again".to_string()
            }))
        };

        match db::get_userdata(&client, &user_token).await {
            Ok(data) => data,
            Err(_) => return Ok(HttpResponse::Ok().json(MessageResponse {
                message: "Failed at retrieving existing data, you may not have your account linked yet".to_string()
            }))
        };

        let updated_data = match db::update_userdata(&client, &user_token, user_data).await {
            Ok(data) => data,
            Err(error) => {
                println!("an update failed: {}", error);
                return Ok(HttpResponse::Ok().json(MessageResponse {
                    message: "The request has unfortunately failed the update".to_string()
                }))
            },
        };

        // TODO: implement role handling
        let roles = format!("The request was successful, you've gained the following roles: {}", "test");

        Ok(HttpResponse::Ok().json(MessageResponse {
            message: roles
        }))
    }
}

use actix_web::{web, App, HttpServer};
use deadpool_postgres::Runtime;
use dotenv::dotenv;
use handlers::update_user;
use tokio_postgres::NoTls;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let config = crate::config::Config::new();
    let pool = config.pg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();

    let server = HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .service(web::resource("/userdata").route(web::post().to(update_user)))
    })
    .bind(config.server_addr.clone())?
    .run();
    println!("Server running at http://{}/", config.server_addr);

    server.await
}