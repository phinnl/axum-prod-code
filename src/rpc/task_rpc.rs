use crate::{
	ctx::Ctx,
	model::{
		task::{Task, TaskBmc, TaskForCreate, TaskForUpdate},
		ModelManager,
	},
};

use super::{ParamsForCreate, ParamsForUpdate, ParamsId, Result};

pub async fn list_tasks(ctx: Ctx, mm: ModelManager) -> Result<Vec<Task>> {
	let tasks = TaskBmc::list(&ctx, &mm).await?;
	Ok(tasks)
}

pub async fn get_task(ctx: Ctx, mm: ModelManager, params: ParamsId) -> Result<Task> {
	let ParamsId { id } = params;
	let task = TaskBmc::get(&ctx, &mm, id).await?;
	Ok(task)
}

pub async fn create_task(
	ctx: Ctx,
	mm: ModelManager,
	params: ParamsForCreate<TaskForCreate>,
) -> Result<Task> {
	let ParamsForCreate { data } = params;
	let task = TaskBmc::create(&ctx, &mm, data).await?;
	Ok(task)
}

pub async fn update_task(
	ctx: Ctx,
	mm: ModelManager,
	params: ParamsForUpdate<TaskForUpdate>,
) -> Result<Task> {
	let ParamsForUpdate { id, data } = params;
	let task = TaskBmc::update(&ctx, &mm, id, data).await?;
	Ok(task)
}

pub async fn delete_task(
	ctx: Ctx,
	mm: ModelManager,
	params: ParamsId,
) -> Result<Task> {
	let ParamsId { id } = params;
	let task = TaskBmc::get(&ctx, &mm, id).await?;
	TaskBmc::delete(&ctx, &mm, id).await?;
	Ok(task)
}
