use crate::crypt::{pwd, EncryptContent};
use crate::ctx::Ctx;
use crate::model::user::{UserBmc, UserForLogin};
use crate::model::ModelManager;
use crate::web::{self, set_token_cookies, Error, Result};
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_cookies::{Cookie, Cookies};
use tracing::{debug, info};

pub fn routes(mm: ModelManager) -> Router {
	Router::new()
		.route("/api/login", post(api_login_handler))
		.with_state(mm)
}

async fn api_login_handler(
	State(mm): State<ModelManager>,
	cookies: Cookies,
	Json(payload): Json<LoginPayload>,
) -> Result<Json<Value>> {
	debug!("{:<12} - api_login_handler", "HANDLER");

	let LoginPayload { username, password } = payload;
	let root_ctx = Ctx::root_ctx();

	let user: UserForLogin = UserBmc::first_by_username(&root_ctx, &mm, &username)
		.await?
		.ok_or(Error::LoginFailUsernameNotFound)?;
	let user_id = user.id;

	let Some(pwd) = user.pwd else {
		return Err(Error::LoginFailUserHasNoPwd { user_id });
	};

	info!(
		"{:<12} - api_login_handler - password: {password}",
		"HANDLER",
	);

	pwd::validate_pwd(
		&EncryptContent {
			content: password,
			salt: user.pwd_salt.to_string(),
		},
		&pwd,
	)
	.map_err(|_| Error::LoginFailPwdNotMatching { user_id })?;

	set_token_cookies(&cookies, &user.username, &user.token_salt.to_string());

	// Create the success body.
	let body = Json(json!({
		"result": {
			"success": true
		}
	}));

	Ok(body)
}

#[derive(Debug, Deserialize)]
struct LoginPayload {
	username: String,
	password: String,
}
