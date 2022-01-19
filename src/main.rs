mod handlers;
pub mod config;
pub mod role_handling;
pub mod constants;
pub mod models;
pub mod errors;
pub mod db;

use actix_web::{web::{self, Data}, App, HttpServer};
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
            .app_data(Data::new(pool.clone()))
            .service(web::resource("/userdata").route(web::post().to(update_user)))
    })
    .bind(config.server_addr.clone())?
    .run();
    println!("Server running at http://{}/", config.server_addr);

    server.await
}
