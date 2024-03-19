// start-region:    --- Modules

use tokio::sync::OnceCell;
use tracing::info;

use crate::{
	ctx::Ctx,
	model::{
		self,
		task::{Task, TaskBmc, TaskForCreate},
		ModelManager,
	},
};

mod dev_db;

// end-region:      --- Modules

/// Initialize environment for local development
/// (for early development, will be called from main())
pub async fn init_dev() {
	static INIT: OnceCell<()> = OnceCell::const_new();

	INIT.get_or_init(|| async {
		info!("{:<12} - mod.rs:15 - dev_init_db()", "FOR DEV ONLY");
		dev_db::init_dev_db().await.unwrap();
	})
	.await;
}

pub async fn init_test() -> ModelManager {
	static INIT: OnceCell<ModelManager> = OnceCell::const_new();

	let mm = INIT
		.get_or_init(|| async {
			info!("{:<12} - mod.rs:27 - test_init_mm()", "FOR TEST ONLY");
			init_dev().await;
			ModelManager::new().await.unwrap()
		})
		.await;

	mm.clone()
}

pub async fn seed_tasks(
  ctx: &Ctx,
  mm: &ModelManager,
	titles: &[&str],
) -> model::Result<Vec<Task>> {
	let mut tasks: Vec<Task> = Vec::new();

	for title in titles {
		let task = TaskBmc::create(
			ctx,
			mm,
			TaskForCreate {
				title: title.to_string(),
			},
		)
		.await?;
		tasks.push(task);
	}

	Ok(tasks)
}
