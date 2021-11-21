use std::collections::HashMap;
use std::str::FromStr;

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

pub trait Printer {
  fn to_raw(&self) -> anyhow::Result<String>;
  fn to_json(&self) -> anyhow::Result<String>;
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Workspace {
  pub id: u64,
  pub name: String,
  pub premium: bool,
  pub admin: bool,
  pub default_hourly_rate: f64,
  pub default_currency: String,
  pub only_admins_may_create_projects: bool,
  pub only_admins_see_billable_rates: bool,
  pub rounding: i8,
  pub rounding_minutes: i8,
  pub at: DateTime<Utc>,
  pub logo_url: Option<String>,
}

impl Printer for Workspace {
  fn to_json(&self) -> anyhow::Result<String> {
    Ok(serde_json::to_string_pretty(&self)?)
  }

  fn to_raw(&self) -> anyhow::Result<String> {
    Ok(self.name.to_string())
  }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Project {
  pub id: u64,
  pub name: String,
  pub wid: u64,
  pub cid: Option<u64>,
  pub active: bool,
  pub is_private: bool,
  pub template: bool,
  pub template_id: Option<u64>,
  pub billable: bool,
  pub auto_estimates: bool,
  pub estimated_hours: Option<u64>,
  pub at: DateTime<Utc>,
  pub color: String,
  pub rate: Option<f64>,
  pub created_at: DateTime<Utc>,
}

impl Printer for Project {
  fn to_json(&self) -> anyhow::Result<String> {
    Ok(serde_json::to_string_pretty(&self)?)
  }

  fn to_raw(&self) -> anyhow::Result<String> {
    Ok(self.name.to_string())
  }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UserData {
  pub id: u64,
  pub api_token: String,
  pub default_wid: u64,
  pub email: String,
  pub fullname: String,
  pub jquery_timeofday_format: String,
  pub jquery_date_format: String,
  pub timeofday_format: String,
  pub date_format: String,
  pub store_start_and_stop_time: bool,
  pub beginning_of_week: u8,
  pub language: String,
  pub image_url: String,
  pub sidebar_piechart: bool,
  pub at: DateTime<Utc>,

  #[serde(default)]
  pub new_blog_post: HashMap<String, String>,
  pub send_product_emails: bool,
  pub send_weekly_report: bool,
  pub send_timer_notifications: bool,
  pub openid_enabled: bool,
  pub timezone: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SinceWith<T> {
  pub since: u64,
  pub data: T,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DataWith<T> {
  pub data: T,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TimeEntry {
  pub id: u64,
  pub wid: u64,
  pub pid: u64,
  pub billable: bool,
  pub start: DateTime<Utc>,
  pub stop: Option<DateTime<Utc>>,
  pub duration: i64,
  pub description: Option<String>,

  #[serde(default)]
  pub tags: Vec<String>,
  pub duronly: bool,
  pub at: DateTime<Utc>,
}

impl Printer for TimeEntry {
  fn to_json(&self) -> anyhow::Result<String> {
    Ok(serde_json::to_string_pretty(&self)?)
  }

  fn to_raw(&self) -> anyhow::Result<String> {
    Ok(self.description.to_owned().unwrap_or_default())
  }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Client {
  pub id: u64,
  pub name: String,
  pub wid: u64,
  pub notes: Option<String>,

  // This shouldn't be an Option:
  // https://github.com/toggl/toggl_api_docs/blob/master/chapters/clients.md#create-a-client
  pub at: Option<DateTime<Utc>>,
}

impl Printer for Client {
  fn to_json(&self) -> anyhow::Result<String> {
    Ok(serde_json::to_string_pretty(&self)?)
  }

  fn to_raw(&self) -> anyhow::Result<String> {
    Ok(self.name.to_string())
  }
}

#[derive(Debug, Clone, Copy)]
pub enum Start {
  Now,
  Date(DateTime<Utc>),
}

impl Start {
  pub fn as_date_time(self) -> DateTime<Utc> {
    match self {
      Start::Now => Utc::now(),
      Self::Date(date) => date,
    }
  }
}

impl FromStr for Start {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "now" => Ok(Start::Now),
      date => Ok(Start::Date(date.parse::<DateTime<Utc>>()?)),
    }
  }
}
