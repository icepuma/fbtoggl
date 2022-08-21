use chrono::DateTime;
use chrono::Datelike;
use chrono::Duration;
use chrono::Local;
use chrono::NaiveDate;
use chrono::TimeZone;
use chrono::Utc;
use chrono::Weekday;
use chronoutil::shift_months;
use now::DateTimeNow;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

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

#[derive(Deserialize, Serialize, Debug)]
pub struct Project {
  pub id: u64,
  pub name: String,
  pub wid: u64,
  pub cid: Option<u64>,
  pub active: bool,
  pub is_private: bool,
  pub template: Option<bool>,
  pub template_id: Option<u64>,
  pub billable: bool,
  pub auto_estimates: bool,
  pub estimated_hours: Option<u64>,
  pub at: DateTime<Utc>,
  pub color: String,
  pub rate: Option<f64>,
  pub created_at: DateTime<Utc>,
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

  // This shouldn't be an Option:
  // https://github.com/toggl/toggl_api_docs/blob/master/chapters/users.md#users
  pub sidebar_piechart: Option<bool>,
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
  pub pid: Option<u64>,
  pub billable: Option<bool>,
  pub start: DateTime<Utc>,
  pub stop: Option<DateTime<Utc>>,
  pub duration: i64,
  pub description: Option<String>,

  #[serde(default)]
  pub tags: Vec<String>,

  #[serde(default)]
  pub duronly: bool,
  pub at: Option<DateTime<Utc>>,
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

#[derive(Debug, Clone, Copy)]
pub enum Range {
  Today,
  Yesterday,
  ThisWeek,
  LastWeek,
  ThisMonth,
  LastMonth,
  FromTo(NaiveDate, NaiveDate),
  Date(NaiveDate),
}

impl Range {
  pub fn get_datetimes(self) -> Vec<DateTime<Local>> {
    let (start, end) = self.as_range();

    // range "today" and "yesterday" have different start and end dates,
    // because toggl.com ranges work like that
    // => return only start date for missing datetime list
    if (end - start).num_days() == 1 {
      return vec![start];
    }

    let mut it = start;
    let mut missing_days = vec![];

    while it <= end {
      let weekday = it.date().weekday();

      if weekday != Weekday::Sat && weekday != Weekday::Sun {
        missing_days.push(it);
      }

      it += Duration::days(1);
    }

    missing_days
  }

  pub fn as_range(self) -> (DateTime<Local>, DateTime<Local>) {
    match self {
      Range::Today => {
        let now = Local::now();
        let start = Local
          .ymd(now.year(), now.month(), now.day())
          .and_hms(0, 0, 0);
        let end = start + Duration::days(1);

        (start, end)
      }
      Range::Yesterday => {
        let now = Local::now() - Duration::days(1);

        let start = Local
          .ymd(now.year(), now.month(), now.day())
          .and_hms(0, 0, 0);
        let end = start + Duration::days(1);

        (start, end)
      }
      Range::ThisWeek => {
        let now = Local::now();

        (now.beginning_of_week(), now.end_of_week())
      }
      Range::LastWeek => {
        let now = Local::now() - Duration::weeks(1);

        (now.beginning_of_week(), now.end_of_week())
      }
      Range::ThisMonth => {
        let now = Local::now();

        (now.beginning_of_month(), now.end_of_month())
      }
      Range::LastMonth => {
        let now = Local::now();

        let date = shift_months(now, -1);

        (date.beginning_of_month(), date.end_of_month())
      }
      Range::FromTo(start_date, end_date) => {
        let start = start_date.and_hms(0, 0, 0);
        let end = end_date.and_hms(0, 0, 0) + Duration::days(1);

        (
          Local.from_local_datetime(&start).unwrap(),
          Local.from_local_datetime(&end).unwrap(),
        )
      }
      Range::Date(date) => {
        let start = Local
          .ymd(date.year(), date.month(), date.day())
          .and_hms(0, 0, 0);
        let end = start + Duration::days(1);

        (start, end)
      }
    }
  }
}

impl FromStr for Range {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "today" => Ok(Range::Today),
      "yesterday" => Ok(Range::Yesterday),
      "this-week" => Ok(Range::ThisWeek),
      "last-week" => Ok(Range::LastWeek),
      "this-month" => Ok(Range::ThisMonth),
      "last-month" => Ok(Range::LastMonth),
      from_to_or_date => match from_to_or_date.find('|') {
        Some(index) => Ok(Range::FromTo(
          NaiveDate::parse_from_str(&from_to_or_date[..index], "%Y-%m-%d")?,
          NaiveDate::parse_from_str(&from_to_or_date[index + 1..], "%Y-%m-%d")?,
        )),
        None => Ok(Range::Date(from_to_or_date.parse()?)),
      },
    }
  }
}

impl Display for Range {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let range = self.as_range();

    let text = format!(
      "{} - {}",
      range.0.format("%Y-%m-%d"),
      range.1.format("%Y-%m-%d")
    );

    write!(f, "{}", text)
  }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Currency {
  pub currency: Option<String>,
  pub amount: Option<f64>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ReportTimeEntry {
  pub id: u64,
  pub pid: u64,
  pub project: String,
  pub client: String,
  pub tid: Option<u64>,
  pub task: Option<String>,
  pub uid: u64,
  pub user: String,
  pub description: String,
  pub start: DateTime<Utc>,
  pub end: DateTime<Utc>,
  pub dur: u64,
  pub updated: DateTime<Utc>,
  pub use_stop: bool,
  pub is_billable: bool,
  pub billable: f64,
  pub cur: String,

  #[serde(default)]
  pub tags: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ReportDetails {
  pub total_grand: Option<u64>,
  pub total_billable: Option<u64>,

  #[serde(default)]
  pub total_currencies: Vec<Currency>,

  pub total_count: u64,
  pub per_page: u64,

  pub data: Vec<ReportTimeEntry>,
}
