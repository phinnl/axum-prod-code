use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};

use crate::{crypt, model::store};

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Debug, Serialize)]
pub enum Error {
  EntityNotFound {
    entity: &'static str,
    id: i64,
  },
  // -- Modules
  Store(store::Error),
  Crypt(crypt::Error),
  // -- External
  Sqlx(#[serde_as(as = "DisplayFromStr")] sqlx::Error),
}

impl From<sqlx::Error> for Error {
  fn from(err: sqlx::Error) -> Self {
    Self::Sqlx(err)
  }
}

impl From<store::Error> for Error {
  fn from(err: store::Error) -> Self {
    Self::Store(err)
  }
}

impl From<crypt::Error> for Error {
  fn from(err: crypt::Error) -> Self {
    Self::Crypt(err)
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