use dotenv::dotenv;

use serde::Deserialize;

use crate::Pool;
use diesel::prelude::PgConnection;
use diesel::r2d2::ConnectionManager;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
}

fn env_var_not_set_msg(env_var: &str) -> String {
    format!(
        "Environment variable `{env_var}` not set",
        env_var = env_var
    )
}

fn env_var_parsing_error_msg(env_var: &str) -> String {
    format!(
        "Environment variable `{env_var}` cannot be parsed",
        env_var = env_var,
    )
}

fn read_from_env<T: std::fmt::Display + std::str::FromStr + std::fmt::Debug>(env_var: &str) -> T {
    match std::env::var(env_var)
        .expect(env_var_not_set_msg(env_var).as_str())
        .parse()
    {
        Ok(parse_value) => parse_value,
        Err(_) => {
            log::error!("{}", env_var_parsing_error_msg(env_var));
            panic!()
        },
    }
}

impl Config {
    pub fn from_env() -> Config {
        dotenv().ok();

        log::info!("Loading configuration");

        let host: String = read_from_env("HOST");
        let port: u16 = read_from_env("PORT");
        let database_url: String = read_from_env("DATABASE_URL");

        Config {
            host: host,
            port: port,
            database_url: database_url,
        }
    }

    pub fn db_bool(&self) -> Pool {
        log::info!("Creating database connection pool.");
        // create db connection Pool
        let manager = ConnectionManager::<PgConnection>::new(&self.database_url);
        r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.".as_ref())
    }

    pub fn address(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}
