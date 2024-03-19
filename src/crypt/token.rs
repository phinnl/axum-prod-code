use std::{fmt::Display, str::FromStr};

use crate::{
	config,
	crypt::{encrypt_into_b64u, EncryptContent},
	utils::{
		b64u_decode, b64u_encode, now_utc, now_utc_plus_secs_str, parse_utc_str,
	},
};

use super::{Error, Result};

// String format: 'ident_b64u.exp_b64u.sign_b64u'
pub struct Token {
	pub ident: String,
	pub exp: String,
	pub sign_b64u: String,
}

impl FromStr for Token {
	type Err = Error;

	fn from_str(token_str: &str) -> std::prelude::v1::Result<Self, Self::Err> {
		let splits = token_str.split('.').collect::<Vec<&str>>();

		if splits.len() != 3 {
			return Err(Error::InvalidTokenFormat);
		}

		let (ident_b64u, exp_b64u, sign_b64u) = (splits[0], splits[1], splits[2]);

		Ok(Self {
			ident: b64u_decode(ident_b64u).map_err(|_| Error::TokenCannotDecode)?,
			exp: b64u_decode(exp_b64u).map_err(|_| Error::TokenCannotDecode)?,
			sign_b64u: sign_b64u.to_string(),
		})
	}
}

impl Display for Token {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}.{}.{}",
			b64u_encode(&self.ident),
			b64u_encode(&self.exp),
			self.sign_b64u
		)
	}
}

// start-region:    --- Private

fn _generate_token(
	ident: &str,
	duration_sec: f64,
	salt: &str,
	key: &[u8],
) -> Result<Token> {
	let ident = ident.to_string();
	let exp = now_utc_plus_secs_str(duration_sec);
	let sign_b64u = _token_sign_into_b64u(&ident, &exp, salt, key)?;
	Ok(Token {
		ident,
		exp,
		sign_b64u,
	})
}

fn _validate_token(token: &Token, salt: &str, key: &[u8]) -> Result<()> {
	let new_sign_b64u = _token_sign_into_b64u(&token.ident, &token.exp, salt, key)?;

	if new_sign_b64u != token.sign_b64u {
		return Err(Error::TokenSignatureNotMatching);
	}

	let exp = parse_utc_str(&token.exp).map_err(|_| Error::TokenExpIsNotIso)?;

  if exp < now_utc() {
		return Err(Error::TokenExpired);
	}

	Ok(())
}

fn _token_sign_into_b64u(
	ident: &str,
	exp: &str,
	salt: &str,
	key: &[u8],
) -> Result<String> {
	let content = format!("{}.{}", b64u_encode(ident), b64u_encode(exp));
	let signature_b64u = encrypt_into_b64u(
		key,
		&EncryptContent {
			content,
			salt: salt.to_string(),
		},
	)?;
	Ok(signature_b64u)
}

// end-region:      --- Private

pub fn generate_web_token(user: &str, salt: &str) -> Result<Token> {
	let config = &config();
	_generate_token(user, config.TOKEN_DURATION, salt, &config.TOKEN_KEY)
}

pub fn validate_web_token(token: &Token, salt: &str) -> Result<()> {
	let token_key = &config().TOKEN_KEY;
	_validate_token(token, salt, token_key)?;
	Ok(())
}

#[cfg(test)]
mod tests {
	use std::{thread, time::Duration};

	use super::*;
	use anyhow::Result;

	#[test]
	fn test_token_display() -> Result<()> {
		let fx_token = Token {
			ident: "fx_ident".to_string(),
			exp: "2024-03-19T07:40:57.548Z".to_string(),
			sign_b64u: "some_sign_b64_encoded".to_string(),
		};
		assert_eq!(
			fx_token.to_string(),
			"ZnhfaWRlbnQ.MjAyNC0wMy0xOVQwNzo0MDo1Ny41NDha.some_sign_b64_encoded"
		);
		Ok(())
	}

	#[test]
	fn test_token_parse_from_str() -> Result<()> {
		let token_str =
			"ZnhfaWRlbnQ.MjAyNC0wMy0xOVQwNzo0MDo1Ny41NDha.some_sign_b64_encoded";
		let fx_token = Token {
			ident: "fx_ident".to_string(),
			exp: "2024-03-19T07:40:57.548Z".to_string(),
			sign_b64u: "some_sign_b64_encoded".to_string(),
		};

		let token = Token::from_str(token_str)?;

		assert!(matches!(token, fx_token));

		Ok(())
	}

	#[test]
	fn test_token_validate_ok() -> Result<()> {
		let fx_user = "fx_user";
		let fx_salt = "fx_salt";
		let fx_duration = 0.01;
		let token_key = &config().TOKEN_KEY;
		let fx_token = _generate_token(fx_user, fx_duration, fx_salt, token_key)?;

		let res = validate_web_token(&fx_token, fx_salt);

		assert!(matches!(res, Ok(())));

		Ok(())
	}

	#[test]
	fn test_token_validate_expired() -> Result<()> {
		let fx_user = "fx_user";
		let fx_salt = "fx_salt";
		let fx_duration = 0.01;
		let token_key = &config().TOKEN_KEY;
		let fx_token = _generate_token(fx_user, fx_duration, fx_salt, token_key)?;

		thread::sleep(Duration::from_millis(20));
		let res = validate_web_token(&fx_token, fx_salt);

		assert!(matches!(res, Err(Error::TokenExpired)), "should be expired {res:?}");

		Ok(())
	}
}
