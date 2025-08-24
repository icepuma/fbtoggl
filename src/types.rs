use core::fmt;
use core::str::FromStr;
use serde::{Deserialize, Serialize};

// Newtype wrappers for IDs to prevent mixing them up
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WorkspaceId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProjectId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClientId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TimeEntryId(pub u64);

impl fmt::Display for WorkspaceId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

impl fmt::Display for ProjectId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

impl fmt::Display for ClientId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

impl fmt::Display for TimeEntryId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

impl FromStr for TimeEntryId {
  type Err = core::num::ParseIntError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    s.parse::<u64>().map(TimeEntryId)
  }
}

// Enum for project status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatus {
  Upcoming,
  Active,
  Ended,
  Archived,
  Deleted,
}

// Enum for client status used in API queries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientStatusFilter {
  Active,
  Both,
}

impl ClientStatusFilter {
  pub const fn as_str(self) -> &'static str {
    match self {
      Self::Active => "active",
      Self::Both => "both",
    }
  }
}

// API Token wrapper for security
#[derive(Clone)]
pub struct ApiToken(String);

impl ApiToken {
  pub fn new(token: String) -> anyhow::Result<Self> {
    if token.is_empty() {
      return Err(anyhow::anyhow!("API token cannot be empty"));
    }
    Ok(Self(token))
  }

  // Only expose the token value when absolutely needed for API calls
  pub(crate) fn as_str(&self) -> &str {
    &self.0
  }
}

impl fmt::Debug for ApiToken {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("ApiToken")
      .field("token", &"[REDACTED]")
      .finish()
  }
}

// Type-safe duration wrapper
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TogglDuration(chrono::Duration);

impl TogglDuration {
  pub fn from_seconds(seconds: i64) -> anyhow::Result<Self> {
    chrono::Duration::try_seconds(seconds)
      .map(Self)
      .ok_or_else(|| anyhow::anyhow!("Invalid duration: {} seconds", seconds))
  }

  pub const fn as_seconds(&self) -> i64 {
    self.0.num_seconds()
  }

  pub const fn inner(&self) -> chrono::Duration {
    self.0
  }

  pub fn is_negative(&self) -> bool {
    self.0 < chrono::Duration::zero()
  }

  pub const fn is_zero(&self) -> bool {
    self.0.is_zero()
  }
}

impl From<chrono::Duration> for TogglDuration {
  fn from(duration: chrono::Duration) -> Self {
    Self(duration)
  }
}

impl From<TogglDuration> for chrono::Duration {
  fn from(duration: TogglDuration) -> Self {
    duration.0
  }
}
