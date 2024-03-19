use tracing::debug;

use crate::{Error, Result};
use std::{env, str::FromStr, sync::OnceLock};

pub fn config() -> &'static Config {
  static INSTANCE: OnceLock<Config> = OnceLock::new();
  INSTANCE.get_or_init(|| {
    Config::load_from_env().unwrap_or_else(|ex| {
      panic!("FATAL - WHILE LOADING CONF : Cause {ex:?}");
    })
  })
}

#[allow(non_snake_case)]
pub struct Config {
  // -- Crypt
  pub PWD_KEY: Vec<u8>,
  pub TOKEN_KEY: Vec<u8>,
  pub TOKEN_DURATION: f64,
	// -- Web
	pub WEB_FOLDER: String,
  // -- Db
	pub DB_URL: String,
}

impl Config {
	fn load_from_env() -> Result<Self> {
		Ok(Self {
      PWD_KEY: get_env_b64u_as_u8s("SERVICE_PWD_KEY")?,
      TOKEN_KEY: get_env_b64u_as_u8s("SERVICE_TOKEN_KEY")?,
      TOKEN_DURATION: get_env_parse("SERVICE_TOKEN_DURATION_SEC")?,
			WEB_FOLDER: get_env("SERVICE_WEB_FOLDER")?,
			DB_URL: get_env("SERVICE_DB_URL")?,
		})
	}
}

fn get_env(name: &'static str) -> Result<String> {
	env::var(name).map_err(|err| Error::ConfigMissingEnv(name))
}

fn get_env_b64u_as_u8s(name: &'static str) -> Result<Vec<u8>> {
  base64_url::decode(&get_env(name)?).map_err(|err| Error::ConfigWrongFormat(name))
}

fn get_env_parse<T: FromStr>(name: &'static str) -> Result<T> {
  get_env(name)?.parse().map_err(|err| Error::ConfigWrongFormat(name))
}
