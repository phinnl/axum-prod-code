use axum::{
	extract::State,
	response::{IntoResponse, Response},
	routing::post,
	Json, Router,
};
use serde::Deserialize;
use serde_json::{from_value, json, to_value, Value};
use tracing::debug;

mod task_rpc;
use crate::{
	ctx::Ctx,
	model::ModelManager,
	web::{Error, Result},
};

#[derive(Deserialize)]
struct RpcRequest {
	id: Option<Value>,
	method: String,
	params: Option<Value>,
}

#[derive(Deserialize)]
pub struct RpcInfo {
	pub id: Option<Value>,
	pub method: String,
}

#[derive(Deserialize)]
pub struct ParamsForCreate<T> {
	pub data: T,
}

#[derive(Deserialize)]
pub struct ParamsForUpdate<T> {
	pub id: i64,
	pub data: T,
}

#[derive(Deserialize)]
pub struct ParamsId {
	pub id: i64,
}

async fn rpc_handler(
	ctx: Ctx,
	State(mm): State<ModelManager>,
	Json(rpc_request): Json<RpcRequest>,
) -> Response {
	let rpc_info = RpcInfo {
		id: rpc_request.id.clone(),
		method: rpc_request.method.clone(),
	};
	let mut res = _rpc_handler(ctx, mm, rpc_request).await.into_response();
	res.extensions_mut().insert(rpc_info);
	res
}

macro_rules! exec_rpc_fn {
	// with params
	($rpc_fn:expr, $ctx:expr, $mm:expr, $rpc_params:expr) => {{
		let rpc_fn_name = stringify!($rpc_fn);
		let params = $rpc_params.ok_or(Error::RpcMissingParams {
			method: rpc_fn_name.to_string(),
		})?;
		let params = from_value(params).map_err(|_| Error::RpcMissingParams {
			method: rpc_fn_name.to_string(),
		})?;
		$rpc_fn($ctx, $mm, params).await.map(to_value)??
	}};
	// without params
	($rpc_fn:expr, $ctx:expr, $mm:expr) => {
		$rpc_fn($ctx, $mm).await.map(to_value)??
	};
}

async fn _rpc_handler(
	ctx: Ctx,
	mm: ModelManager,
	rpc_request: RpcRequest,
) -> Result<Json<Value>> {
	let RpcRequest { id, method, params } = rpc_request;

	debug!("{:<12} - mod.rs:50 - method: {method}", "HANDLER");

	let result = match method.as_str() {
		"list_tasks" => exec_rpc_fn!(task_rpc::list_tasks, ctx, mm),
		"get_task" => exec_rpc_fn!(task_rpc::get_task, ctx, mm, params),
		"create_task" => exec_rpc_fn!(task_rpc::create_task, ctx, mm, params),
		"update_task" => exec_rpc_fn!(task_rpc::update_task, ctx, mm, params),
		"delete_task" => exec_rpc_fn!(task_rpc::delete_task, ctx, mm, params),
		_ => return Err(Error::RpcMethodNotFound(method)),
	};

	Ok(Json(json!({
	  "id": id,
	  "method": method,
	  "result": result,
	})))
}

pub fn routes(mm: ModelManager) -> Router {
	Router::new()
		.route("/rpc", post(rpc_handler))
		.with_state(mm)
}
