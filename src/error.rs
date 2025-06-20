//! Custom error types for the Toggl API client
//!
//! This module provides domain-specific error types for better error handling
//! and more informative error messages.

#![allow(
  clippy::std_instead_of_core,
  reason = "std::error::Error trait is not yet available in core"
)]

use core::fmt;

/// Represents errors that can occur when interacting with the Toggl API
#[derive(Debug)]
pub enum TogglError {
  /// Authentication failed (401)
  Authentication(String),

  /// Access forbidden (403)
  Forbidden(String),

  /// Resource not found (404)
  NotFound { resource: String },

  /// Invalid request (400)
  BadRequest(String),

  /// Rate limit exceeded (429)
  RateLimit { retry_after: Option<u64> },

  /// Server error (5xx)
  ServerError { status: u16, message: String },

  /// Network error
  Network(minreq::Error),

  /// JSON parsing error
  Json(serde_json::Error),

  /// URL parsing error
  Url(url::ParseError),

  /// Other errors
  Other(anyhow::Error),
}

impl fmt::Display for TogglError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Authentication(msg) => write!(f, "Authentication failed: {msg}"),
      Self::Forbidden(msg) => write!(f, "Access forbidden: {msg}"),
      Self::NotFound { resource } => {
        write!(f, "Resource not found: {resource}")
      }
      Self::BadRequest(msg) => write!(f, "Invalid request: {msg}"),
      Self::RateLimit { retry_after } => match retry_after {
        Some(seconds) => {
          write!(f, "Rate limit exceeded, retry after {seconds} seconds")
        }
        None => write!(f, "Rate limit exceeded"),
      },
      Self::ServerError { status, message } => {
        write!(f, "Server error ({status}): {message}")
      }
      Self::Network(err) => write!(f, "Network error: {err}"),
      Self::Json(err) => write!(f, "JSON error: {err}"),
      Self::Url(err) => write!(f, "URL error: {err}"),
      Self::Other(err) => write!(f, "{err}"),
    }
  }
}

impl std::error::Error for TogglError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      Self::Network(err) => Some(err),
      Self::Json(err) => Some(err),
      Self::Url(err) => Some(err),
      Self::Other(err) => Some(err.as_ref()),
      _ => None,
    }
  }
}

impl From<minreq::Error> for TogglError {
  fn from(err: minreq::Error) -> Self {
    Self::Network(err)
  }
}

impl From<serde_json::Error> for TogglError {
  fn from(err: serde_json::Error) -> Self {
    Self::Json(err)
  }
}

impl From<url::ParseError> for TogglError {
  fn from(err: url::ParseError) -> Self {
    Self::Url(err)
  }
}

impl From<anyhow::Error> for TogglError {
  fn from(err: anyhow::Error) -> Self {
    Self::Other(err)
  }
}

/// Convert HTTP status codes to appropriate `TogglError` variants
pub fn from_status_code(status: u16, body: &str, service: &str) -> TogglError {
  match status {
    401 => TogglError::Authentication(format!("{service} API: {body}")),
    403 => TogglError::Forbidden(format!("{service} API: {body}")),
    404 => TogglError::NotFound {
      resource: body.to_owned(),
    },
    400 => TogglError::BadRequest(body.to_owned()),
    429 => {
      // Try to parse retry-after from response
      TogglError::RateLimit { retry_after: None }
    }
    500..=599 => TogglError::ServerError {
      status,
      message: body.to_owned(),
    },
    _ => TogglError::Other(anyhow::anyhow!(
      "{service} API error ({status}): {body}"
    )),
  }
}

/// Result type alias for Toggl operations
pub type Result<T> = core::result::Result<T, TogglError>;
