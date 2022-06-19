#[macro_use]
extern crate diesel;
use crate::{auth::actor::AuthManager, config::Config, db::actor::DbActor};
use actix::{Addr, SyncArbiter};
use actix_cors::Cors;
use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use dotenv::dotenv;
use std::cmp::max;

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
    // App configuration
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

    // Worker allocation
    let num_of_cpus = num_cpus::get();
    let server_workers = max(num_of_cpus >> 1, 1);
    let db_actor_workers = max((((num_of_cpus - server_workers) as f32) * 0.7).floor() as usize, 1);
    let auth_mgr_workers = max(num_of_cpus - server_workers - db_actor_workers, 1);

    log::info!("Number of Logical Cores: {}", num_of_cpus);
    log::info!("Worker Allocation:");
    log::info!("Num Server Workers: {}", server_workers);
    log::info!("Num DB Workers: {}", db_actor_workers);
    log::info!("Num Auth Mgr Workers: {}", auth_mgr_workers);

    // Spawning workers
    let db_actor_addr = SyncArbiter::start(db_actor_workers, move || db_actor.clone());
    let auth_mgr_addr = SyncArbiter::start(auth_mgr_workers, move || auth_mgr.clone());

    let app_state = AppState {
        db_actor_addr: db_actor_addr,
        auth_mgr_addr: auth_mgr_addr,
    };

    log::info!("Starting server on: http://{}:{}", host, port);
    HttpServer::new(move || {
        let cors = cors(config.cors_allow_all);
        App::new()
            .wrap(cors)
            .app_data(Data::new(app_state.clone()))
            .wrap(Logger::default()) // enable logger
            .configure(views::users::config)
            .configure(views::posts::config)
            .configure(views::swagger_ui::config)
    })
    .workers(server_workers)
    .bind((host, port))?
    .run()
    .await?;

    Ok(())
}
