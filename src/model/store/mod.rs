// start-region:    --- Modules

mod error;

use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub use self::error::{Error, Result};

use crate::config;

// end-region:      --- Modules

pub type Db = Pool<Postgres>;

pub async fn new_db_pool() -> Result<Db> {
	PgPoolOptions::new()
		.max_connections(5)
		.connect(&config().DB_URL)
		.await
		.map_err(|err| Error::FailToCreatePool(err.to_string()))
}
