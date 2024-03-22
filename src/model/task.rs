use modql::{
	field::Fields,
	filter::{FilterNodes, ListOptions, OpValsBool, OpValsInt64, OpValsString},
};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use super::{Error, ModelManager, Result};
use crate::ctx::Ctx;

use super::base::{self, DbBmc};

#[derive(Clone, Debug, FromRow, Fields, Serialize)]
pub struct Task {
	id: i64,
	title: String,
	done: bool,
}

#[derive(Deserialize, Fields)]
pub struct TaskForCreate {
	pub title: String,
}

#[derive(Deserialize, Fields, Default)]
pub struct TaskForUpdate {
	title: Option<String>,
	done: Option<bool>,
}

#[derive(FilterNodes, Deserialize, Default, Debug)]
pub struct TaskFilter {
	id: Option<OpValsInt64>,
	title: Option<OpValsString>,
	done: Option<OpValsBool>,
}

pub struct TaskBmc;

impl DbBmc for TaskBmc {
	const TABLE: &'static str = "task";
}

impl TaskBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		payload: TaskForCreate,
	) -> Result<Task> {
		base::create::<Self, _, TaskForCreate>(ctx, mm, payload).await
	}

	pub async fn update(
		ctx: &Ctx,
		mm: &ModelManager,
		id: i64,
		payload: TaskForUpdate,
	) -> Result<Task> {
		base::update::<Self, _, TaskForUpdate>(ctx, mm, id, payload).await
	}

	pub async fn get(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<Task> {
		base::get::<Self, _>(ctx, mm, id).await
	}

	pub async fn list(
		ctx: &Ctx,
		mm: &ModelManager,
		filters: Option<Vec<TaskFilter>>,
		list_options: Option<ListOptions>,
	) -> Result<Vec<Task>> {
		base::list::<Self, _, _>(ctx, mm, filters, list_options).await
	}

	pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()> {
		base::delete::<Self>(ctx, mm, id).await
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::_dev_utils;
	use anyhow::Result;
	use axum::routing::delete;
	use serial_test::serial;

	#[serial]
	#[tokio::test]
	async fn test_task_create_task() -> Result<()> {
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();
		let fx_title = "test_task_create_title";

		// Execute
		let task = TaskBmc::create(
			&ctx,
			&mm,
			TaskForCreate {
				title: fx_title.to_string(),
			},
		)
		.await?;
		let id = task.id;

		// Check
		assert_eq!(fx_title, task.title);

		// Cleanup
		TaskBmc::delete(&ctx, &mm, id).await?;

		Ok(())
	}

	#[serial]
	#[tokio::test]
	async fn test_task_update_task() -> Result<()> {
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();
		let fx_title = "test_task_create_title";

		// Execute
		let task = TaskBmc::create(
			&ctx,
			&mm,
			TaskForCreate {
				title: fx_title.to_string(),
			},
		)
		.await?;
		let id = task.id;

		// Check
		assert_eq!(fx_title, task.title);

		// Update
		let task = TaskBmc::update(
			&ctx,
			&mm,
			id,
			TaskForUpdate {
				title: Some("test_task_update_title".to_string()),
				..Default::default()
			},
		)
		.await?;
		assert_eq!("test_task_update_title", task.title);

		// Cleanup
		TaskBmc::delete(&ctx, &mm, id).await?;

		Ok(())
	}

	#[serial]
	#[tokio::test]
	async fn test_task_get_not_found() -> Result<()> {
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();
		let fx_id = 100;

		// Execute
		let task = TaskBmc::get(&ctx, &mm, fx_id).await;

		// Check
		assert!(
			matches!(task, Err(Error::EntityNotFound { entity: "task", id })),
			"Entity not found matching"
		);

		Ok(())
	}

	#[serial]
	#[tokio::test]
	async fn test_task_delete_not_found() -> Result<()> {
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();
		let fx_id = 100;

		// Execute
		let task = TaskBmc::delete(&ctx, &mm, fx_id).await;

		// Check
		assert!(
			matches!(task, Err(Error::EntityNotFound { entity: "task", id })),
			"Entity not found matching"
		);

		Ok(())
	}

	#[serial]
	#[tokio::test]
	async fn test_task_list() -> Result<()> {
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();
		let fx_titles = ["test_task_title_1", "test_task_title_2"];
		let tasks = _dev_utils::seed_tasks(&ctx, &mm, &fx_titles).await?;

		// Execute
		let tasks = TaskBmc::list(&ctx, &mm, None, None).await?;

		// Check
		let tasks = tasks
			.into_iter()
			.filter(|task| task.title.starts_with("test_task_title"))
			.collect::<Vec<Task>>();
		assert_eq!(tasks.len(), 2, "number of seeded tasks");

		// Cleanup
		for task in tasks {
			TaskBmc::delete(&ctx, &mm, task.id).await?;
		}

		Ok(())
	}
}
