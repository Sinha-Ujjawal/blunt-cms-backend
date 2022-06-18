#[macro_use]
extern crate diesel;
use crate::{auth::actor::AuthManager, config::Config, db::actor::DbActor};
use actix::{Addr, SyncArbiter};
use actix_cors::Cors;
use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use dotenv::dotenv;

mod argon2_password_hasher;
mod auth;
mod config;
mod db;
mod errors;
mod openapi;
mod views;

#[derive(Clone)]
pub struct AppState {
    db_actor_addr: Addr<DbActor>,
    auth_mgr_addr: Addr<AuthManager>,
}

fn cors(cors_allow_all: u8) -> Cors {
    if cors_allow_all == 1 {
        log::info!("Allowing Any Origin");
        Cors::permissive()
    } else {
        log::info!("Default Cors Setup");
        Cors::default()
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = Config::from_env();

    let host = config.host;
    let port = config.port;

    let db_actor = DbActor::new(&config.database_url);
    let auth_mgr = AuthManager::new(
        config.jwt_secret,
        config.jwt_expiration_duration,
        config.redis_server_url,
        config.redis_server_get_connection_timeout,
    );

    let db_actor_addr = SyncArbiter::start(5, move || db_actor.clone());
    let auth_mgr_addr = SyncArbiter::start(5, move || auth_mgr.clone());

    let app_state = AppState {
        db_actor_addr: db_actor_addr,
        auth_mgr_addr: auth_mgr_addr,
    };

    log::info!("Started server on: http://{}:{}", host, port);

    HttpServer::new(move || {
        let cors = cors(config.cors_allow_all);
        App::new()
            .wrap(cors)
            .app_data(Data::new(app_state.clone()))
            .wrap(Logger::default()) // enable logger
            .configure(views::users::config)
            .configure(views::posts::config)
            .configure(views::drafts::config)
            .configure(views::swagger_ui::config)
    })
    .bind((host, port))?
    .run()
    .await?;

    Ok(())
}
