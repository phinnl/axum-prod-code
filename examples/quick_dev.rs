#![allow(unused)] // For beginning only.

use anyhow::Result;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
	let hc = httpc_test::new_client("http://localhost:8080")?;

	let req_login = hc.do_post(
		"/api/login",
		json!({
		"username": "demo1",
			  "password": "demo_pwd"
		  }),
	);

	let req_tasks =
		hc.do_post("/api/rpc", json!({ "id": 1,"method": "list_tasks" }));
	let req_get_task = hc.do_post(
		"/api/rpc",
		json!({ "id": 1,"method": "get_task", "params": { "id": 1 } }),
	);

	let req_create_task =
		hc.do_post("/api/rpc", json!({ "id": 1,"method": "create_task", "params": { "data": { "title": "first task from rpc" } } }));

	req_login.await?.print().await?;
	req_get_task.await?.print().await?;

	let req_tasks = hc.do_post("/api/rpc", json!({ "id": 1,"method": "list_tasks" }));
	// req_create_task.await?.print().await?;
	req_tasks.await?.print().await?;
	Ok(())
}
