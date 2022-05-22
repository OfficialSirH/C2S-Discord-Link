mod handlers;
pub mod config;
pub mod role_handling;
pub mod constants;
pub mod models;
pub mod errors;
pub mod db;
pub mod webhook_logging;

use actix_web::{web::{self, Data}, App, HttpServer};
use deadpool_postgres::Runtime;
use dotenv::dotenv;
use webhook_logging::webhook_log;
use tokio_postgres::NoTls;

use crate::handlers::{delete_user, create_user, update_user};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let config = crate::config::Config::new();
    let pool = config.pg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();

    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(web::resource("/userdata").route(web::post().to(create_user)))
            // TODO: remove the post to update_user after the game hits stable with the new API
            .service(web::resource("/userdata").route(web::post().to(update_user)))
            .service(web::resource("/userdata/:playerId").route(web::patch().to(update_user)))
            .service(web::resource("/userdata/:playerId").route(web::delete().to(delete_user)))
    })
    .bind(config.server_addr.clone())?
    .run();
    webhook_log(format!("Server running at http://{}/", config.server_addr), constants::LOG::SUCCESSFUL).await;
    println!("Server running at http://{}/", config.server_addr);

    server.await
}
