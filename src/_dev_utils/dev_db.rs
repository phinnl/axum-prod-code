use std::{fs, path::PathBuf, time::Duration};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tracing::info;

type Db = Pool<Postgres>;

// NOTE: hardcode to prevent deployed system db update
const PG_DEV_POSTGRES_URL: &str =
	"postgres://postgres:welcome@localhost:5432/postgres";
const PG_DEV_APP_URL: &str =
	"postgres://app_user:dev_only_password@localhost:5432/app_db";

// sql files
const SQL_RECREATE_DB: &str = "sql/dev_initial/00-recreate-db.sql";
const SQL_DIR: &str = "sql/dev_initial";

pub async fn init_dev_db() -> Result<(), sqlx::Error> {
	info!("{:<12} - dev_db.rs:21", "FOR DEV ONLY");

	// -- Create the app_db/app_user with the postgres user
	{
		let root_db = new_db_pool(PG_DEV_POSTGRES_URL).await?;
		pexec(&root_db, SQL_RECREATE_DB).await?;
	}

	// -- Get sql files
	let mut paths: Vec<PathBuf> = fs::read_dir(SQL_DIR)?
		.filter_map(|entry| entry.ok().map(|e| e.path()))
		.collect();
  // sort path by filename, ex: [01-recreate-db.sql, 02-create-tables.sql, ...]
  paths.sort();

  let app_db = new_db_pool(PG_DEV_APP_URL).await?;
  for path in paths {
    if let Some(path) = path.to_str() {
      // replace path for window
      let path = path.replace('\\', "/");

      // Only take sql file and skip the SQL_RECREATE_DB
      if path.ends_with(".sql") && path != SQL_RECREATE_DB {
        pexec(&app_db, &path).await?;
      }
    }
  }

	Ok(())
}

async fn pexec(db: &Db, file: &str) -> Result<(), sqlx::Error> {
	info!("{:<12} - dev_db.rs:21 - {file}", "FOR DEV ONLY");

	// --- Read the file
	let content = fs::read_to_string(file)?;

	// FIXME: Make the split more sql proof.
	let sqls: Vec<&str> = content.split(';').collect();

	for sql in sqls {
		sqlx::query(sql).execute(db).await?;
	}

	Ok(())
}

async fn new_db_pool(db_con_url: &str) -> Result<Db, sqlx::Error> {
	PgPoolOptions::new()
		.max_connections(1)
		.acquire_timeout(Duration::from_millis(500))
		.connect(db_con_url)
		.await
}