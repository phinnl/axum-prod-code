use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::{
	ctx::Ctx,
	model::{ModelManager, Result},
};

#[derive(Clone, Debug, FromRow, Serialize)]
pub struct Task {
	id: i64,
	title: String,
}

#[derive(Deserialize)]
pub struct TaskForCreate {
	title: String,
}

#[derive(Deserialize)]
pub struct TaskForUpdate {
	title: Option<String>,
}

pub struct TaskBmc;

impl TaskBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		payload: TaskForCreate,
	) -> Result<Task> {
		let db = mm.db();
		let task = sqlx::query_as::<_, Task>(
			"INSERT INTO task (title) VALUES ($1) RETURNING *",
		)
		.bind(payload.title)
		.fetch_one(db)
		.await?;
		Ok(task)
	}
}

#[cfg(test)]
mod tests {
	use crate::_dev_utils;
	use super::*;
	use anyhow::Result;

	#[tokio::test]
	async fn test_create_task() -> Result<()> {
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();
		let fx_title = "test_task_title";
		let task = TaskBmc::create(
			&ctx,
			&mm,
			TaskForCreate {
				title: fx_title.to_string(),
			},
		)
		.await?;
		println!("->> {:<12} - task.rs:64 - {task:?}", "HANDLER");

		let id = task.id;

		// check
		let (title,): (String,) =
			sqlx::query_as("SELECT title FROM task WHERE id = $1")
				.bind(id)
				.fetch_one(mm.db())
				.await?;
		assert_eq!(fx_title, title);

		// cleanup
		let count = sqlx::query("DELETE FROM task WHERE id = $1")
			.bind(id)
			.execute(mm.db())
			.await?
			.rows_affected();
		assert_eq!(1, count, "Deleted 1 row, id = {id}");
		Ok(())
	}
}
