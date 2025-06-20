use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use minreq::{Method, Request};
use url::Url;

pub const CREATED_WITH: &str = "fbtoggl (https://github.com/icepuma/fbtoggl)";
pub const AUTHORIZATION: &str = "Authorization";

pub fn basic_auth(api_token: &str) -> (String, String) {
  (
    AUTHORIZATION.to_owned(),
    format!(
      "Basic {}",
      STANDARD.encode(format!("{api_token}:api_token"))
    ),
  )
}

pub fn base_request(
  base_url: &Url,
  method: Method,
  uri: &str,
  api_token: &str,
) -> anyhow::Result<Request> {
  let full_url = base_url.join(uri)?;
  let (key, value) = basic_auth(api_token);
  Ok(minreq::Request::new(method, full_url).with_header(key, value))
}

use colored::Colorize;

pub fn print_request_debug(
  request: &Request,
  body: Option<&serde_json::Value>,
) {
  println!("{}", "Request:".bold().underline());

  // Print the full request using Debug trait
  // This will include method, URL, headers (including Authorization), etc.
  println!("{request:?}");

  // If there's a body, print it as well
  if let Some(body) = body {
    println!(
      "Body: {}",
      serde_json::to_string_pretty(body).unwrap_or_else(|_| body.to_string())
    );
  }

  println!();
}
