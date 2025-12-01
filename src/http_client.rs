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
  /// Get the base URL for this client
  fn base_url(&self) -> &Url;

  /// Get the API token for authentication
  fn api_token(&self) -> &ApiToken;

  /// Get the service name for error messages
  fn service_name(&self) -> &'static str;

  /// Create a base request with authentication
  fn base_request(&self, method: Method, uri: &str) -> Result<Request> {
    base_request(self.base_url(), method, uri, self.api_token().as_str())
      .map_err(TogglError::from)
  }

  /// Make a request without a body and return the raw response
  fn raw_request(
    &self,
    debug: bool,
    method: Method,
    uri: &str,
  ) -> Result<Response> {
    let request = self.base_request(method, uri)?;

    if debug {
      print_request_debug(&request, None);
    }

    request.send().map_err(TogglError::from)
  }

  /// Make a request with a JSON body and return the raw response
  fn raw_request_with_json(
    &self,
    debug: bool,
    method: Method,
    uri: &str,
    body: &serde_json::Value,
  ) -> Result<Response> {
    let request = self.base_request(method, uri)?.with_json(body)?;

    if debug {
      print_request_debug(&request, Some(body));
    }

    request.send().map_err(TogglError::from)
  }
}

/// Extension trait for type-safe request/response handling
pub trait HttpClientExt: HttpClient {
  /// Make a request without a body and return the deserialized response
  fn request<D: DeserializeOwned + Debug>(
    &self,
    debug: bool,
    method: Method,
    uri: &str,
  ) -> Result<D> {
    let response = self.raw_request(debug, method, uri)?;
    handle_response(debug, response, self.service_name())
  }

  /// Make a request without a body and expect an empty response
  fn empty_request(
    &self,
    debug: bool,
    method: Method,
    uri: &str,
  ) -> Result<()> {
    let response = self.raw_request(debug, method, uri)?;
    handle_empty_response(response, self.service_name())
  }

  /// Make a request with a JSON body and return the deserialized response
  fn request_with_body<D: DeserializeOwned + Debug, S: Serialize + Debug>(
    &self,
    debug: bool,
    method: Method,
    uri: &str,
    body: S,
  ) -> Result<D> {
    let json_body = serde_json::to_value(body)?;
    let response =
      self.raw_request_with_json(debug, method, uri, &json_body)?;
    handle_response(debug, response, self.service_name())
  }
}

// Implement the extension trait for all types that implement HttpClient
impl<T: HttpClient> HttpClientExt for T {}

/// Handle a response that should contain data
#[allow(
  clippy::needless_pass_by_value,
  reason = "Response is consumed by .json() and .as_str() methods"
)]
#[allow(
  clippy::cast_possible_truncation,
  clippy::cast_sign_loss,
  clippy::as_conversions,
  reason = "HTTP status codes are guaranteed to be positive and fit in u16"
)]
fn handle_response<D: DeserializeOwned + Debug>(
  debug: bool,
  response: Response,
  service: &str,
) -> Result<D> {
  if debug {
    println!("{}", "Response:".bold().underline());
    println!("{response:?}");
    println!();
  }

  match response.status_code {
    200 | 201 if debug => match response.json() {
      Ok(json) => {
        println!("{}", "Received JSON response:".bold().underline());
        println!("{json:?}");
        println!();
        Ok(json)
      }
      Err(err) => Err(TogglError::Other(anyhow::anyhow!("JSON error: {err}"))),
    },
    200 | 201 => response
      .json()
      .map_err(|e| TogglError::Other(anyhow::anyhow!("JSON error: {e}"))),
    status => response.as_str().map_or_else(
      |_| {
        Err(from_status_code(
          status as u16,
          "Unable to read response body",
          service,
        ))
      },
      |text| Err(from_status_code(status as u16, text, service)),
    ),
  }
}

/// Handle a response that should be empty
#[allow(
  clippy::needless_pass_by_value,
  reason = "Response is consumed by .as_str() method"
)]
#[allow(
  clippy::cast_possible_truncation,
  clippy::cast_sign_loss,
  clippy::as_conversions,
  reason = "HTTP status codes are guaranteed to be positive and fit in u16"
)]
fn handle_empty_response(response: Response, service: &str) -> Result<()> {
  match response.status_code {
    200 | 201 => Ok(()),
    status => response.as_str().map_or_else(
      |_| {
        Err(from_status_code(
          status as u16,
          "Unable to read response body",
          service,
        ))
      },
      |text| Err(from_status_code(status as u16, text, service)),
    ),
  }
}

/// Extension trait for handling responses with headers
pub trait ResponseExt {
  /// Handle a response and extract a specific header value along with the body
  fn handle_with_header<D: DeserializeOwned + Debug>(
    self,
    debug: bool,
    header_name: &str,
    service: &str,
  ) -> Result<(D, Option<String>)>;
}

impl ResponseExt for Response {
  #[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::as_conversions,
    reason = "HTTP status codes are guaranteed to be positive and fit in u16"
  )]
  fn handle_with_header<D: DeserializeOwned + Debug>(
    self,
    debug: bool,
    header_name: &str,
    service: &str,
  ) -> Result<(D, Option<String>)> {
    if debug {
      println!("{}", "Response:".bold().underline());
      println!("{self:?}");
      println!();
    }

    let header_value = self.headers.get(header_name).cloned();

    match self.status_code {
      200 | 201 if debug => match self.json() {
        Ok(json) => {
          println!("{}", "Received JSON response:".bold().underline());
          println!("{json:?}");
          println!();
          Ok((json, header_value))
        }
        Err(err) => {
          Err(TogglError::Other(anyhow::anyhow!("JSON error: {err}")))
        }
      },
      200 | 201 => Ok((
        self
          .json()
          .map_err(|e| TogglError::Other(anyhow::anyhow!("JSON error: {e}")))?,
        header_value,
      )),
      status => self.as_str().map_or_else(
        |_| {
          Err(from_status_code(
            status as u16,
            "Unable to read response body",
            service,
          ))
        },
        |text| Err(from_status_code(status as u16, text, service)),
      ),
    }
  }
}
