use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use derive_more::From;

use crate::{crypt, model::store};

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Debug, Serialize, From)]
pub enum Error {
  EntityNotFound {
    entity: &'static str,
    id: i64,
  },
  UpdateFailed {
    entity: &'static str,
    id: i64,
  },
  // -- Modules
  #[from]
  Store(store::Error),
  #[from]
  Crypt(crypt::Error),
  // -- External
  #[from]
  Sqlx(#[serde_as(as = "DisplayFromStr")] sqlx::Error),
  #[from]
  SeaQuery(#[serde_as(as = "DisplayFromStr")] sea_query::error::Error),

  #[from]
  ModqlIntoSea(#[serde_as(as = "DisplayFromStr")] modql::filter::IntoSeaError),

  ListLimitExceeded {
    max: i64,
    actual: i64,
  }
}

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
	fn fmt(
		&self,
		fmt: &mut core::fmt::Formatter,
	) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for Error {}
// endregion: --- Error Boilerplate