use crate::{
    db,
    errors::MyError,
    models::{MessageResponse, ReceivedUserData},
    role_handling::handle_roles,
};
use actix_web::{web, HttpResponse};
use deadpool_postgres::{Client, Pool};
use serde::Deserialize;
use crypto::{sha1::Sha1, hmac::Hmac, mac::Mac};

trait ConvertResultErrorToMyError<T> {
    fn make_response(self: Self, error_enum: MyError) -> Result<T, MyError>;
}

impl<T, E: std::fmt::Debug> ConvertResultErrorToMyError<T> for Result<T, E> {
    fn make_response(self: Self, error_enum: MyError) -> Result<T, MyError> {
        match self {
            Ok(data) => Ok(data),
            Err(error) => {
                println!("{:?}", error);
                Err(error_enum)
            }
        }
    }
}

#[derive(Deserialize)]
pub struct PlayerData {
    #[serde(rename = "playerId")]
    player_id: String,
}

pub async fn update_user(
    query: web::Query<PlayerData>,
    received_user: web::Json<ReceivedUserData>,
    db_pool: web::Data<Pool>,
) -> Result<HttpResponse, MyError> {
    let user_data: ReceivedUserData = received_user.into_inner();

    let client: Client = db_pool.get().await.make_response(MyError::InternalError(
        "request failed at creating database client, please try again",
    ))?;
    let config = crate::config::Config::new();

    let mut user_token = Hmac::new(Sha1::new(), &config.userdata_auth.as_bytes());
    user_token.input(query.player_id.as_bytes());
    user_token.input(user_data.player_token.as_bytes());

    let user_token = user_token
        .result()
        .code()
        .iter()
        .map(|byte| format!("{:02x?}", byte))
        .collect::<Vec<String>>()
        .join("");

    db::get_userdata(&client, &user_token)
        .await
        .make_response(MyError::InternalError(
            "Failed at retrieving existing data, you may not have your account linked yet",
        ))?;

    let updated_data = db::update_userdata(&client, &user_token, user_data)
        .await
        .make_response(MyError::InternalError(
            "The request has unfortunately failed the update",
        ))?;

    let gained_roles =
        handle_roles(updated_data, config)
            .await
            .make_response(MyError::InternalError(
                "The role-handling process has failed",
            ))?;
    let roles = format!(
        "The request was successful, you've gained the following roles: {}",
        gained_roles.join(", ")
    );

    Ok(HttpResponse::Ok().json(MessageResponse { message: roles }))
}
