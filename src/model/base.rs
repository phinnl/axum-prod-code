use super::{Error, ModelManager, Result};
use crate::ctx::Ctx;
use sqlb::HasFields;
use sqlx::{postgres::PgRow, FromRow};

pub trait DbBmc {
	const TABLE: &'static str;
}

pub async fn get<MC, E>(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
where
	MC: DbBmc,
	E: for<'r> FromRow<'r, PgRow> + Unpin + Send + HasFields,
{
	let db = mm.db();
	let entity = sqlb::select()
		.table(MC::TABLE)
		.columns(E::field_names())
		.and_where("id", "=", id)
		.fetch_optional(db)
		.await?
		.ok_or(Error::EntityNotFound {
			entity: MC::TABLE,
			id,
		})?;
	Ok(entity)
}

pub async fn list<MC, E>(ctx: &Ctx, mm: &ModelManager) -> Result<Vec<E>>
where
	MC: DbBmc,
	E: for<'r> FromRow<'r, PgRow> + Unpin + Send + HasFields,
{
	let db = mm.db();
	let entities = sqlb::select()
		.table(MC::TABLE)
		.columns(E::field_names())
		.fetch_all(db)
		.await?;
	Ok(entities)
}

pub async fn create<MC, E, EC>(
	ctx: &Ctx,
	mm: &ModelManager,
	payload: EC,
) -> Result<E>
where
	MC: DbBmc,
	E: for<'r> FromRow<'r, PgRow> + Unpin + Send + HasFields,
	EC: HasFields,
{
	let db = mm.db();
	let entity = sqlb::insert()
		.table(MC::TABLE)
		.data(payload.not_none_fields())
		.returning(E::field_names())
		.fetch_one(db)
		.await?;
	Ok(entity)
}

pub async fn delete<MC>(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()>
where
	MC: DbBmc,
{
	let db = mm.db();
	let count = sqlb::delete()
		.table(MC::TABLE)
		.and_where("id", "=", id)
		.exec(db)
		.await?;
	if count != 1 {
		return Err(Error::EntityNotFound {
			entity: MC::TABLE,
			id,
		});
	}
	Ok(())
}

pub async fn update<MC, E, EU>(
	ctx: &Ctx,
	mm: &ModelManager,
	id: i64,
	payload: EU,
) -> Result<E>
where
	MC: DbBmc,
	E: for<'r> FromRow<'r, PgRow> + Unpin + Send + HasFields,
	EU: HasFields,
{
	let db = mm.db();
	let entity = sqlb::update()
		.table(MC::TABLE)
		.and_where("id", "=", id)
		.data(payload.not_none_fields())
		.returning(E::field_names())
		.fetch_one(db)
		.await?;
	Ok(entity)
}
