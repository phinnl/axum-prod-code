use super::{Error, ModelManager, Result};
use crate::ctx::Ctx;
use modql::{
	field::HasFields,
	filter::{FilterGroups, ListOptions},
	SIden,
};
use sea_query::{
	Condition, Expr, Iden, IntoIden, PostgresQueryBuilder, Query, ReturningClause,
	TableRef,
};
use sea_query_binder::SqlxBinder;
use sqlx::{postgres::PgRow, FromRow};

#[derive(Iden)]
enum CommonIden {
	Id,
}

pub trait DbBmc {
	const TABLE: &'static str;

	fn table_ref() -> TableRef {
		TableRef::Table(SIden(Self::TABLE).into_iden())
	}
}

const LIST_LIMIT_MAX: i64 = 100;
const LIST_LIMIT_DEFAULT: i64 = 20;

pub fn finalize_list_options(
	list_options: Option<ListOptions>,
) -> Result<ListOptions> {
	if let Some(mut list_options) = list_options {
		if let Some(limit) = list_options.limit {
			if limit > LIST_LIMIT_MAX {
				return Err(Error::ListLimitExceeded {
					max: LIST_LIMIT_MAX,
					actual: limit,
				});
			}
		} else {
			list_options.limit = Some(LIST_LIMIT_DEFAULT);
		}
		return Ok(list_options);
	}
	Ok(ListOptions {
		limit: Some(LIST_LIMIT_DEFAULT),
		offset: None,
		order_bys: Some("id".into()),
	})
}

pub async fn get<MC, E>(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
where
	MC: DbBmc,
	E: for<'r> FromRow<'r, PgRow> + Unpin + Send + HasFields,
{
	let db = mm.db();

	// build query
	let mut query = Query::select();
	query
		.from(MC::table_ref())
		.columns(E::field_column_refs())
		.and_where(Expr::col(CommonIden::Id).eq(id));
	let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

	// exec query
	let entity = sqlx::query_as_with::<_, E, _>(&sql, values)
		.fetch_optional(db)
		.await?
		.ok_or(Error::EntityNotFound {
			entity: MC::TABLE,
			id,
		})?;

	Ok(entity)
}

pub async fn list<MC, E, F>(
	ctx: &Ctx,
	mm: &ModelManager,
	filters: Option<F>,
	list_options: Option<ListOptions>,
) -> Result<Vec<E>>
where
	MC: DbBmc,
	E: for<'r> FromRow<'r, PgRow> + Unpin + Send + HasFields,
	F: Into<FilterGroups>,
{
	let db = mm.db();
	// build query
	let mut query = Query::select();
	query.from(MC::table_ref()).columns(E::field_column_refs());

	if let Some(filters) = filters {
		let filters: FilterGroups = filters.into();
		let condition: Condition = filters.try_into()?;
		query.cond_where(condition);
	}

	let list_options = finalize_list_options(list_options)?;
	list_options.apply_to_sea_query(&mut query);

	let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

	let entities = sqlx::query_as_with::<_, E, _>(&sql, values)
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
	// extract fields
	let fields = payload.not_none_fields();
	let (columns, sea_values) = fields.for_sea_insert();

	// build query
	let mut query = Query::insert();
	query
		.into_table(MC::table_ref())
		.columns(columns)
		.values(sea_values)?
		.returning(ReturningClause::All);
	let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

	// exec query
	let entity = sqlx::query_as_with::<_, E, _>(&sql, values)
		.fetch_one(db)
		.await?;

	Ok(entity)
}

pub async fn delete<MC>(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()>
where
	MC: DbBmc,
{
	let db = mm.db();

	// build query
	let mut query = Query::delete();
	query
		.from_table(MC::table_ref())
		.and_where(Expr::col(CommonIden::Id).eq(id));

	// exec query
	let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
	let count = sqlx::query_with(&sql, values)
		.execute(db)
		.await?
		.rows_affected();
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
	// extract fields
	let fields = payload.not_none_fields();
	let fields = fields.for_sea_update();

	// build query
	let mut query = Query::update();
	query
		.table(MC::table_ref())
		.values(fields)
		.and_where(Expr::col(CommonIden::Id).eq(id))
		.returning(ReturningClause::All);
	let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

	// exec query
	let entity = sqlx::query_as_with::<_, E, _>(&sql, values)
		.fetch_one(db)
		.await?;

	Ok(entity)
}
