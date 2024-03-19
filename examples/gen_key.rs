use anyhow::Result;
use rand::{thread_rng, RngCore};

fn main() -> Result<()> {
	let mut key = [0u8; 64];
	thread_rng().fill_bytes(&mut key);
	println!("\nGenerated key for HMAC:\n{key:?}");
	let b64u = base64_url::encode(&key);
	println!("\nKey b64 encoded:\n{b64u}");
	Ok(())
}
