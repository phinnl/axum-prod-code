// region:    --- Modules

mod error;
pub mod mw_auth;
pub mod mw_res_map;
pub mod routes_login;
pub mod routes_static;

use tower_cookies::{Cookie, Cookies};

use crate::crypt::token::generate_web_token;

pub use self::error::ClientError;
pub use self::error::{Error, Result};

// endregion: --- Modules

pub const AUTH_TOKEN: &str = "auth-token";

pub fn set_token_cookies(cookies: &Cookies, user: &str, salt: &str) -> Result<()> {
	let token = generate_web_token(user, salt)?;
	let mut token_cookie = Cookie::new(AUTH_TOKEN, token.to_string());
	token_cookie.set_http_only(true);
	token_cookie.set_path("/");

	cookies.add(token_cookie);

	Ok(())
}

pub fn remove_token_cookies(cookies: &Cookies) -> Result<()> {
	let mut token_cookie = Cookie::named(AUTH_TOKEN);
	token_cookie.set_path("/");

	cookies.remove(token_cookie);
  
	Ok(())
}
