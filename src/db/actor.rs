use actix::{Actor, SyncContext};

use diesel::prelude::PgConnection;
use diesel::r2d2::ConnectionManager;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbPoolConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

#[derive(Clone)]
pub struct DbActor {
    db_pool: DbPool,
}

impl DbActor {
    fn db_pool_result<'a>(database_url: &'a str) -> Result<DbPool, r2d2::Error> {
        log::info!("Creating Database connection pool.");
        // create redis connection pook
        let manager = ConnectionManager::new(database_url);
        r2d2::Pool::builder().build(manager) // Aborts if `min_idle` is greater than `max_size`. Need to think about retry
    }

    pub fn new<'a>(database_url: &'a str) -> Self {
        let db_pool =
            Self::db_pool_result(database_url).expect("Error creating database pool connection!");
        DbActor { db_pool: db_pool }
    }

    pub fn get_conn_result(&self) -> Result<DbPoolConnection, r2d2::Error> {
        self.db_pool.get()
    }

    pub fn get_conn(&self) -> DbPoolConnection {
        self.get_conn_result().expect("Cannot connect to database!")
    }
}

impl Actor for DbActor {
    type Context = SyncContext<Self>;
}
