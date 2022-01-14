use crate::{db, errors::MyError, models::ReceivedUserData, role_handling::handle_roles};
use actix_web::{web, Error, HttpResponse};
use deadpool_postgres::{Client, Pool};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use std::str::from_utf8;

#[derive(Serialize)]
struct MessageResponse {
    message: String,
}

type HmacSha256 = Hmac<sha2::Sha256>;

#[allow(non_snake_case)]
#[derive(Deserialize)]
pub struct PlayerData {
    playerId: String,
    playerToken: String,
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
        Err(_) => {
            return Ok(HttpResponse::Ok().json(MessageResponse {
                message: "request failed at token-creation process, please try again".to_string(),
            }))
        }
    };
    mac.update(query.playerId.as_bytes());
    mac.update(query.playerToken.as_bytes());

    let user_token = mac.finalize().into_bytes();
    let user_token = match from_utf8(&user_token) {
        Ok(token) => token,
        Err(_) => {
            return Ok(HttpResponse::Ok().json(MessageResponse {
                message: "request failed at token-creation process, please try again".to_string(),
            }))
        }
    };

    match db::get_userdata(&client, &user_token).await {
        Ok(data) => data,
        Err(_) => {
            return Ok(HttpResponse::Ok().json(MessageResponse {
                message:
                    "Failed at retrieving existing data, you may not have your account linked yet"
                        .to_string(),
            }))
        }
    };

    let updated_data = match db::update_userdata(&client, &user_token, user_data).await {
        Ok(data) => data,
        Err(error) => {
            println!("an update failed: {}", error);
            return Ok(HttpResponse::Ok().json(MessageResponse {
                message: "The request has unfortunately failed the update".to_string(),
            }));
        }
    };

    // TODO: implement role handling
    let gained_roles = handle_roles(updated_data, config);
    let roles = format!(
        "The request was successful, you've gained the following roles: {}",
        "test"
    );

    Ok(HttpResponse::Ok().json(MessageResponse { message: roles }))
}
