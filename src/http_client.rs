//! Shared HTTP client trait and utilities for Toggl API clients
//!
//! This module provides a common trait and shared functionality for both
//! the main Toggl API client and the Reports API client.

use crate::common::{base_request, print_request_debug};
use crate::error::{Result, TogglError, from_status_code};
use crate::types::ApiToken;
use colored::Colorize;
use core::fmt::Debug;
use minreq::{Method, Request, Response};
use serde::{Serialize, de::DeserializeOwned};
use url::Url;

/// Common trait for Toggl API clients - contains only non-generic methods
pub trait HttpClient {
  fn base_url(&self) -> &Url;
  fn api_token(&self) -> &ApiToken;
  fn service_name(&self) -> &'static str;
  fn debug(&self) -> bool;

  fn base_request(&self, method: Method, uri: &str) -> Result<Request> {
    base_request(self.base_url(), method, uri, self.api_token().as_str())
      .map_err(TogglError::from)
  }
}

/// Extension trait for type-safe request/response handling
pub trait HttpClientExt: HttpClient {
  /// GET-style request without a body; deserialize the JSON response.
  fn request<D: DeserializeOwned + Debug>(
    &self,
    method: Method,
    uri: &str,
  ) -> Result<D> {
    let response = send(self, method, uri, None)?;
    decode_json(self.debug(), response, self.service_name())
  }

  /// Request with no expected body in the response (e.g. DELETE).
  fn empty_request(&self, method: Method, uri: &str) -> Result<()> {
    let response = send(self, method, uri, None)?;
    check_status(response, self.service_name()).map(|_| ())
  }

  /// Request with a JSON body; deserialize the JSON response.
  fn request_with_body<D: DeserializeOwned + Debug, S: Serialize + Debug>(
    &self,
    method: Method,
    uri: &str,
    body: S,
  ) -> Result<D> {
    let json_body = serde_json::to_value(body)?;
    let response = send(self, method, uri, Some(&json_body))?;
    decode_json(self.debug(), response, self.service_name())
  }
}

impl<T: HttpClient> HttpClientExt for T {}

fn send<C: HttpClient + ?Sized>(
  client: &C,
  method: Method,
  uri: &str,
  body: Option<&serde_json::Value>,
) -> Result<Response> {
  let request = match body {
    Some(b) => client.base_request(method, uri)?.with_json(b)?,
    None => client.base_request(method, uri)?,
  };

  if client.debug() {
    print_request_debug(&request, body);
  }

  request.send().map_err(TogglError::from)
}

/// Return the response on 2xx, or a typed error on anything else.
pub fn check_status(response: Response, service: &str) -> Result<Response> {
  match response.status_code {
    200 | 201 => Ok(response),
    status => Err(response.as_str().map_or_else(
      |_| from_status_code(status, "Unable to read response body", service),
      |text| from_status_code(status, text, service),
    )),
  }
}

#[allow(
  clippy::needless_pass_by_value,
  reason = "Response is consumed by .json() and .as_str()"
)]
fn decode_json<D: DeserializeOwned + Debug>(
  debug: bool,
  response: Response,
  service: &str,
) -> Result<D> {
  if debug {
    println!("{}", "Response:".bold().underline());
    println!("{response:?}");
    println!();
  }

  let response = check_status(response, service)?;
  let json: D = response
    .json()
    .map_err(|e| TogglError::Other(anyhow::anyhow!("JSON error: {e}")))?;

  if debug {
    println!("{}", "Received JSON response:".bold().underline());
    println!("{json:?}");
    println!();
  }

  Ok(json)
}
