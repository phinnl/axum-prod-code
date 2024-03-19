use super::{base, Result};
use serde::{Deserialize, Serialize};
use sqlb::{Fields, HasFields};
use sqlx::{postgres::PgRow, prelude::FromRow};
use uuid::Uuid;

use crate::{
	crypt::{pwd, EncryptContent},
	ctx::Ctx,
};

use super::{base::DbBmc, ModelManager};

#[derive(Clone, Debug, Fields, FromRow, Serialize)]
pub struct User {
	pub id: i64,
	pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct UserForCreate {
	pub username: String,
	pub password: String,
}

#[derive(Fields)]
struct UserForInsert {
	username: String,
}

#[derive(Clone, Debug, Fields, FromRow)]
pub struct UserForLogin {
	pub id: i64,
	pub username: String,
	pub pwd: Option<String>,
	pub pwd_salt: Uuid,
	pub token_salt: Uuid,
}

#[derive(Clone, Debug, Fields, FromRow)]
pub struct UserForAuth {
	pub id: i64,
	pub username: String,
	pub token_salt: Uuid,
}

pub trait UserBy: HasFields + for<'r> FromRow<'r, PgRow> + Unpin + Send {}

impl UserBy for User {}
impl UserBy for UserForLogin {}
impl UserBy for UserForAuth {}

pub struct UserBmc;

impl DbBmc for UserBmc {
	const TABLE: &'static str = "user";
}

impl UserBmc {
	pub async fn get<U>(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<U>
	where
		U: UserBy,
	{
		base::get::<Self, _>(ctx, mm, id).await
	}

	pub async fn first_by_username<U>(
		ctx: &Ctx,
		mm: &ModelManager,
		username: &str,
	) -> Result<Option<U>>
	where
		U: UserBy,
	{
		let db = mm.db();
		let user = sqlb::select()
			.table(Self::TABLE)
			.and_where("username", "=", username)
			.fetch_optional::<_, U>(db)
			.await?;
		Ok(user)
	}

	pub async fn update_pwd(
		ctx: &Ctx,
		mm: &ModelManager,
		id: i64,
		pwd_clear: &str,
	) -> Result<()> {
		let db = mm.db();
		let user: UserForLogin = Self::get(ctx, mm, id).await?;
		let pwd = pwd::encrypt_pwd(&EncryptContent {
			content: pwd_clear.to_string(),
			salt: user.pwd_salt.to_string(),
		})?;

		sqlb::update()
			.and_where("id", "=", id)
			.data(vec![("pwd", pwd).into()])
			.table(Self::TABLE)
			.exec(db)
			.await?;

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::_dev_utils;
	use anyhow::Result;
	use serial_test::serial;

	#[serial]
	#[tokio::test]
	async fn test_get_demo_user() -> Result<()> {
		let ctx = Ctx::root_ctx();
		let mm = _dev_utils::init_test().await;
		let demo_user = User {
			id: 1000,
			username: "demo1".to_string(),
		};

		// Get root user
		let user = UserBmc::get::<User>(&ctx, &mm, demo_user.id).await?;

		// Check
		assert!(matches!(user, demo_user));

		Ok(())
	}

	#[serial]
	#[tokio::test]
	async fn test_get_demo_user_by_username() -> Result<()> {
		let ctx = Ctx::root_ctx();
		let mm = _dev_utils::init_test().await;
		let demo_user = User {
			id: 1000,
			username: "demo1".to_string(),
		};

		// Get root user
		let user =
			UserBmc::first_by_username::<User>(&ctx, &mm, &demo_user.username)
				.await?;

		// Check
		assert!(matches!(user, Some(demo_user)));

		Ok(())
	}
}
