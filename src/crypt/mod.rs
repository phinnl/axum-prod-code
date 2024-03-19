mod error;

pub mod pwd;
pub use error::{Error, Result};
use hmac::{Hmac, Mac};
use sha2::Sha512;

pub struct EncryptContent {
	pub content: String, // clear content
	pub salt: String,    // clear salt
}

pub fn encrypt_into_b64u(
	key: &[u8],
	encrypt_content: &EncryptContent,
) -> Result<String> {
	let EncryptContent { content, salt } = encrypt_content;

	let mut hmac_sha512 =
		Hmac::<Sha512>::new_from_slice(key).map_err(|_| Error::KeyFailHmac)?;
	hmac_sha512.update(content.as_bytes());
	hmac_sha512.update(salt.as_bytes());

	let hmac_result = hmac_sha512.finalize();
	let result_bytes = hmac_result.into_bytes();
	let result = base64_url::encode(&result_bytes);

	Ok(result)
}

#[cfg(test)]
mod tests {
	use super::*;
	use anyhow::Result;
	use rand::{thread_rng, RngCore};

	#[test]
	fn test_encrypt_into_b64u_ok() -> Result<()> {
		let mut fx_key = [0u8; 64];
		thread_rng().fill_bytes(&mut fx_key);
		let fx_encrypt_content = EncryptContent {
			content: String::from("hello world"),
			salt: String::from("hello salt"),
		};
		let fx_res = encrypt_into_b64u(&fx_key, &fx_encrypt_content)?;
		// exec
		let res = encrypt_into_b64u(&fx_key, &fx_encrypt_content)?;

		// check
		assert_eq!(fx_res, res);
    
		Ok(())
	}
}
