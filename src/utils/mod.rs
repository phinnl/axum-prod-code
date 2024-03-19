mod error;

use time::{format_description::well_known::Rfc3339, OffsetDateTime};

pub use self::error::{Error, Result};

pub fn now_utc() -> OffsetDateTime {
	OffsetDateTime::now_utc()
}

pub fn format_time(time: OffsetDateTime) -> String {
	time.format(&Rfc3339).unwrap()
}

pub fn now_utc_plus_secs_str(secs: f64) -> String {
	format_time(now_utc() + time::Duration::seconds_f64(secs))
}

pub fn parse_utc_str(s: &str) -> Result<OffsetDateTime> {
	OffsetDateTime::parse(s, &Rfc3339)
		.map_err(|_| Error::DateFailParse(s.to_string()))
}

pub fn b64u_encode(content: &str) -> String {
	base64_url::encode(content)
}

pub fn b64u_decode(b64_content: &str) -> Result<String> {
	base64_url::decode(b64_content)
		.ok()
		.and_then(|r| String::from_utf8(r).ok())
		.ok_or(Error::FailToDecodeB64u)
}
