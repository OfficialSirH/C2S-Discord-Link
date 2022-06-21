use crate::{
    constants::{ErrorLogType, LOG},
    db,
    errors::MyError,
    models::{MessageResponse, OGUpdateUserData, UpdateUserData},
    role_handling::handle_roles,
    webhook_logging::webhook_log,
};
use actix_web::{web, HttpResponse};
use async_trait::async_trait;
use crypto::{hmac::Hmac, mac::Mac, sha1::Sha1};
use deadpool_postgres::{Client, Pool};
use serde::Deserialize;

trait ConvertResultErrorToMyError<T> {
    fn make_response(self, error_enum: MyError) -> Result<T, MyError>;
}

#[async_trait]
trait LogMyError<T> {
    async fn make_log(self, error_type: ErrorLogType) -> Result<T, MyError>;
}

impl<T, E: std::fmt::Debug> ConvertResultErrorToMyError<T> for Result<T, E> {
    fn make_response(self, error_enum: MyError) -> Result<T, MyError> {
        match self {
            Ok(data) => Ok(data),
            Err(error) => {
                println!("{:?}", error);
                Err(error_enum)
            }
        }
    }
}

#[async_trait]
impl<T: std::marker::Send> LogMyError<T> for Result<T, MyError> {
    async fn make_log(self, error_type: ErrorLogType) -> Result<T, MyError> {
        match self {
            Ok(value) => Ok(value),
            Err(error) => {
                let error_content = match error_type {
                    ErrorLogType::USER(token) => format!(
                        "Error with a user\n\ntoken: {}\n\n{}",
                        token,
                        error.to_string()
                    ),
                    ErrorLogType::INTERNAL => error.to_string(),
                };
                webhook_log(error_content, LOG::FAILURE).await;
                return Err(error);
            }
        }
    }
}

#[derive(Deserialize)]
pub struct PlayerData {
    #[serde(rename = "playerId")]
    player_id: String,
}

pub async fn create_user_pathway(
    pool: web::Data<Pool>,
    // og_update_user params
    query: Option<web::Query<PlayerData>>,
    received_user: Option<web::Json<OGUpdateUserData>>,
    // create_user params
    create_user_data: Option<web::Json<UpdateUserData>>,
) -> Result<HttpResponse, MyError> {
    match query {
        Some(q) => match received_user {
            Some(user) => og_update_user(q, user, pool).await,
            None => Err(MyError::BadRequest("No user data received")),
        },
        None => create_user(create_user_data, pool).await,
    }
}

pub async fn og_update_user(
    query: web::Query<PlayerData>,
    received_user: web::Json<OGUpdateUserData>,
    db_pool: web::Data<Pool>,
) -> Result<HttpResponse, MyError> {
    let user_data = received_user.into_inner();

    println!("og update user function");

    let client: Client = db_pool
        .get()
        .await
        .make_response(MyError::InternalError(
            "request failed at creating database client, please try again",
        ))
        .make_log(ErrorLogType::INTERNAL)
        .await?;
    let config = crate::config::Config::new();

    let mut user_token = Hmac::new(Sha1::new(), config.userdata_auth.as_bytes());
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
        ))
        .make_log(ErrorLogType::USER(user_token.to_string()))
        .await?;

    let updated_data = db::update_userdata(&client, &user_token, UpdateUserData::from(user_data))
        .await
        .make_response(MyError::InternalError(
            "The request has unfortunately failed the update",
        ))
        .make_log(ErrorLogType::USER(user_token.to_string()))
        .await?;

    let gained_roles = handle_roles(&updated_data, config)
        .await
        .make_response(MyError::InternalError(
            "The role-handling process has failed",
        ))
        .make_log(ErrorLogType::USER(user_token))
        .await?;
    let roles = if gained_roles.join(", ").is_empty() {
        "The request was successful, but you've already gained all of the possible roles with your current progress".to_string()
    } else {
        format!(
            "The request was successful, you've gained the following roles: {}",
            gained_roles.join(", ")
        )
    };

    let logged_roles = if gained_roles.join(", ").is_empty() {
        format!(
            "user with ID {} had a successful request but gained no roles",
            updated_data.discord_id
        )
    } else {
        format!(
            "user with ID {} gained the following roles: {}",
            updated_data.discord_id,
            gained_roles.join(", ")
        )
    };

    webhook_log(logged_roles, LOG::INFORMATIONAL).await;
    Ok(HttpResponse::Ok().json(MessageResponse { message: roles }))
}

pub async fn update_user(
    received_user: web::Json<UpdateUserData>,
    db_pool: web::Data<Pool>,
) -> Result<HttpResponse, MyError> {
    let user_data = received_user.into_inner();

    let client: Client = db_pool
        .get()
        .await
        .make_response(MyError::InternalError(
            "request failed at creating database client, please try again",
        ))
        .make_log(ErrorLogType::INTERNAL)
        .await?;
    let config = crate::config::Config::new();

    let mut user_token = Hmac::new(Sha1::new(), config.userdata_auth.as_bytes());
    // user_token.input(query.player_id.as_bytes());
    // TODO: create a helper function to simplify the process of using the authorization docoding.
    // user_token.input(user_data.player_token.as_bytes());

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
        ))
        .make_log(ErrorLogType::USER(user_token.to_string()))
        .await?;

    let updated_data = db::update_userdata(&client, &user_token, user_data)
        .await
        .make_response(MyError::InternalError(
            "The request has unfortunately failed the update",
        ))
        .make_log(ErrorLogType::USER(user_token.to_string()))
        .await?;

    let gained_roles = handle_roles(&updated_data, config)
        .await
        .make_response(MyError::InternalError(
            "The role-handling process has failed",
        ))
        .make_log(ErrorLogType::USER(user_token))
        .await?;
    let roles = if gained_roles.join(", ").is_empty() {
        "The request was successful, but you've already gained all of the possible roles with your current progress".to_string()
    } else {
        format!(
            "The request was successful, you've gained the following roles: {}",
            gained_roles.join(", ")
        )
    };

    let logged_roles = if gained_roles.join(", ").is_empty() {
        format!(
            "user with ID {} had a successful request but gained no roles",
            updated_data.discord_id
        )
    } else {
        format!(
            "user with ID {} gained the following roles: {}",
            updated_data.discord_id,
            gained_roles.join(", ")
        )
    };

    webhook_log(logged_roles, LOG::INFORMATIONAL).await;
    Ok(HttpResponse::Ok().json(MessageResponse { message: roles }))
}

pub async fn create_user(
    received_user: Option<web::Json<UpdateUserData>>,
    db_pool: web::Data<Pool>,
) -> Result<HttpResponse, MyError> {
    // let _user_data = received_user.into_inner();

    println!("create user function");

    let client: Client = db_pool
        .get()
        .await
        .make_response(MyError::InternalError(
            "request failed at creating database client, please try again",
        ))
        .make_log(ErrorLogType::INTERNAL)
        .await?;
    let config = crate::config::Config::new();

    let mut user_token = Hmac::new(Sha1::new(), config.userdata_auth.as_bytes());
    // TODO: get the middleware auth validation done so then it can be used for this function.
    // user_token.input(query.player_id.as_bytes());
    // user_token.input(user_data.player_token.as_bytes());

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
        ))
        .make_log(ErrorLogType::USER(user_token.to_owned()))
        .await?;

    Ok(HttpResponse::Ok().json(MessageResponse {
        message: "The request was successful (WIP POST route)".to_owned(),
    }))

    // TODO: rewrite this area to create an entry instead.
    // let updated_data = db::update_userdata(&client, &user_token, user_data)
    //     .await
    //     .make_response(MyError::InternalError(
    //         "The request has unfortunately failed the update",
    //     ))
    //     .make_log(ErrorLogType::USER(user_token.to_string()))
    //     .await?;

    // let gained_roles = handle_roles(&updated_data, config)
    //     .await
    //     .make_response(MyError::InternalError(
    //         "The role-handling process has failed",
    //     ))
    //     .make_log(ErrorLogType::USER(user_token))
    //     .await?;
    // let roles = if gained_roles.join(", ").is_empty() {
    //     "The request was successful, but you've already gained all of the possible roles with your current progress".to_string()
    // } else {
    //     format!(
    //         "The request was successful, you've gained the following roles: {}",
    //         gained_roles.join(", ")
    //     )
    // };

    // let logged_roles = if gained_roles.join(", ").is_empty() {
    //     format!(
    //         "user with ID {} had a successful request but gained no roles",
    //         updated_data.discord_id
    //     )
    // } else {
    //     format!(
    //         "user with ID {} gained the following roles: {}",
    //         updated_data.discord_id,
    //         gained_roles.join(", ")
    //     )
    // };

    // webhook_log(logged_roles, LOG::INFORMATIONAL).await;
    // Ok(HttpResponse::Ok().json(MessageResponse { message: roles }))
}

pub async fn delete_user(db_pool: web::Data<Pool>) -> Result<HttpResponse, MyError> {
    let client: Client = db_pool
        .get()
        .await
        .make_response(MyError::InternalError(
            "request failed at creating database client, please try again",
        ))
        .make_log(ErrorLogType::INTERNAL)
        .await?;
    let config = crate::config::Config::new();

    let mut user_token = Hmac::new(Sha1::new(), config.userdata_auth.as_bytes());
    // user_token.input(query.player_id.as_bytes());
    // user_token.input(user_data.player_token.as_bytes());

    let user_token = user_token
        .result()
        .code()
        .iter()
        .map(|byte| format!("{:02x?}", byte))
        .collect::<Vec<String>>()
        .join("");

    // db::get_userdata(&client, &user_token)
    //     .await
    //     .make_response(MyError::InternalError(
    //         "Failed at retrieving existing data, you may not have your account linked yet",
    //     ))
    //     .make_log(ErrorLogType::USER(user_token.to_string()))
    //     .await?;

    Ok(HttpResponse::Ok().json(MessageResponse {
        message: "The request was successful (WIP DELETE route)".to_owned(),
    }))

    // TODO: rewrite this area to delete an entry instead.
    // let updated_data = db::update_userdata(&client, &user_token, user_data)
    //     .await
    //     .make_response(MyError::InternalError(
    //         "The request has unfortunately failed the update",
    //     ))
    //     .make_log(ErrorLogType::USER(user_token.to_string()))
    //     .await?;

    // let gained_roles = handle_roles(&updated_data, config)
    //     .await
    //     .make_response(MyError::InternalError(
    //         "The role-handling process has failed",
    //     ))
    //     .make_log(ErrorLogType::USER(user_token))
    //     .await?;
    // let roles = if gained_roles.join(", ").is_empty() {
    //     "The request was successful, but you've already gained all of the possible roles with your current progress".to_string()
    // } else {
    //     format!(
    //         "The request was successful, you've gained the following roles: {}",
    //         gained_roles.join(", ")
    //     )
    // };

    // let logged_roles = if gained_roles.join(", ").is_empty() {
    //     format!(
    //         "user with ID {} had a successful request but gained no roles",
    //         updated_data.discord_id
    //     )
    // } else {
    //     format!(
    //         "user with ID {} gained the following roles: {}",
    //         updated_data.discord_id,
    //         gained_roles.join(", ")
    //     )
    // };

    // webhook_log(logged_roles, LOG::INFORMATIONAL).await;
    // Ok(HttpResponse::Ok().json(MessageResponse { message: roles }))
}
