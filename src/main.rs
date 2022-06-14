#[macro_use]
extern crate diesel;
use crate::config::Config;
use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use dotenv::dotenv;

mod config;
mod errors;
mod models;
mod schema;
mod selectors;
mod services;
mod utils;
mod views;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = Config::from_env();
    let db_pool = config.db_pool();
    let auth_mgr = config.auth_mgr();

    log::info!("Started server on: {}", config.address());

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(db_pool.clone()))
            .app_data(Data::new(auth_mgr.clone()))
            .wrap(Logger::default()) // enable logger
            .configure(views::users::config)
            .configure(views::posts::config)
            .configure(views::drafts::config)
    })
    .bind((config.host, config.port))?
    .run()
    .await?;

    Ok(())
}
