use super::{base, Error, Result};
use modql::field::{Fields, HasFields};
use sea_query::{Expr, Iden, IntoIden, PostgresQueryBuilder, Query, SimpleExpr};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
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

#[derive(Iden)]
enum UserIden {
	Id,
	Username,
	Pwd,
}

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
		// build query
		let mut query = Query::select();
		query
			.from(Self::table_ref())
			.columns(U::field_column_refs())
			.and_where(Expr::col(UserIden::Username).eq(username));

		// exec query
		let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
		let user = sqlx::query_as_with::<_, U, _>(&sql, values)
			.fetch_optional(db)
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

		// encrypt pwd
		let pwd = pwd::encrypt_pwd(&EncryptContent {
			content: pwd_clear.to_string(),
			salt: user.pwd_salt.to_string(),
		})?;

		// build query
		let mut query = Query::update();
		query
			.table(Self::table_ref())
			.value(UserIden::Pwd, SimpleExpr::from(pwd))
			.and_where(Expr::col(UserIden::Id).eq(id));

		// exec query
		let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
		let count = sqlx::query_with(&sql, values)
			.execute(db)
			.await?
			.rows_affected();

    if count != 1 {
      return Err(Error::UpdateFailed {
        entity: Self::TABLE,
        id,
      });
    }

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

	#[serial]
	#[tokio::test]
	async fn test_update_pwd_demo() -> Result<()> {
		let ctx = Ctx::root_ctx();
		let mm = _dev_utils::init_test().await;
		let fx_user = User {
			id: 1000,
			username: "demo1".to_string(),
		};
    let fx_pwd = "123456";
    
    // Update pwd
    UserBmc::update_pwd(&ctx, &mm, fx_user.id, fx_pwd).await?;
    
		// Get root user
		let user =
			UserBmc::first_by_username::<UserForLogin>(&ctx, &mm, &fx_user.username)
      .await?.unwrap();

    let fx_pwd_encrypted = pwd::encrypt_pwd(&EncryptContent {
      content: fx_pwd.to_string(),
      salt: user.pwd_salt.to_string(),
    })?;

		// Check
		assert!(matches!(fx_pwd_encrypted, fx_pwd));

		Ok(())
	}
}
