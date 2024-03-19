use tracing::info;

use crate::{config, crypt::encrypt_into_b64u};

use super::{EncryptContent, Error, Result};

pub fn encrypt_pwd(encrypt_content: &EncryptContent) -> Result<String> {
  let key = &config().PWD_KEY;
  let pwd_b64u = encrypt_into_b64u(key, encrypt_content)?;
  Ok(format!("#01#{pwd_b64u}"))
}

pub fn validate_pwd(encrypt_content: &EncryptContent, pwd: &str) -> Result<()> {
  let key = &config().PWD_KEY;
  let pwd_b64u = encrypt_pwd(encrypt_content)?;

  if pwd != pwd_b64u {
    return Err(Error::PwdNotMatching);
  }
  Ok(())
}