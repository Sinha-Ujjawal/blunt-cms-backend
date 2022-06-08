// dependencies
#[macro_use]
extern crate diesel;
use crate::config::Config;
use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use diesel::prelude::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;

// module declarations
mod config;
mod errors;
mod models;
mod schema;
mod selectors;
mod services;
mod views;

// type declarations
pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = Config::from_env();

    let pool = config.db_bool();

    log::info!("Started server on: {:?}", config.address());

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(pool.clone()))
            .wrap(Logger::default()) // enable logger
            .configure(views::users::config)
    })
    .bind((config.host, config.port))?
    .run()
    .await?;

    Ok(())
}
