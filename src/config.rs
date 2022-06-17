use dotenv::dotenv;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_duration: u32,
    pub redis_server_url: String,
    pub redis_server_get_connection_timeout: u64,
    pub cors_allow_all: u8,
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
        }
    }
}

impl Config {
    pub fn from_env() -> Config {
        dotenv().ok();
        log::info!("Loading configuration");

        let host: String = read_from_env("HOST");
        let port: u16 = read_from_env("PORT");
        let database_url: String = read_from_env("DATABASE_URL");
        let jwt_secret: String = read_from_env("JWT_SECRET");
        let jwt_expiration_duration: u32 = read_from_env("JWT_EXPIRATION_DURATION");
        let redis_server_url: String = read_from_env("REDIS_SERVER_URL");
        let redis_server_get_connection_timeout: u64 =
            read_from_env("REDIS_SERVER_GET_CONNECTION_TIMEOUT");
        let cors_allow_all: u8 = read_from_env("CORS_ALLOW_ALL");

        Config {
            host: host,
            port: port,
            database_url: database_url,
            jwt_secret: jwt_secret,
            jwt_expiration_duration: jwt_expiration_duration,
            redis_server_url: redis_server_url,
            redis_server_get_connection_timeout: redis_server_get_connection_timeout,
            cors_allow_all: cors_allow_all,
        }
    }
}
