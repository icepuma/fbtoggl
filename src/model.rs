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
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

#[derive(Deserialize, Serialize, Debug)]
pub struct Workspace {
  pub id: u64,
  pub name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Project {
  pub id: u64,
  pub name: String,
  pub wid: u64,
  pub status: String,
  pub cid: Option<u64>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Me {
  pub default_workspace_id: u64,
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
  pub tags: Option<Vec<String>>,

  #[serde(default)]
  pub duronly: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Client {
  pub id: u64,
  pub name: String,
  pub archived: bool,
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
  pub fn get_datetimes(self) -> anyhow::Result<Vec<DateTime<Local>>> {
    let (start, end) = self.as_range()?;

    // range "today" and "yesterday" have different start and end dates,
    // because toggl.com ranges work like that
    // => return only start date for missing datetime list
    if (end - start).num_days() == 1 {
      return Ok(vec![start]);
    }

    let mut it = start;
    let mut missing_days = vec![];

    while it <= end {
      let weekday = it.date_naive().weekday();

      if weekday != Weekday::Sat && weekday != Weekday::Sun {
        missing_days.push(it);
      }

      it += Duration::try_days(1).unwrap();
    }

    Ok(missing_days)
  }

  pub fn as_range(self) -> anyhow::Result<(DateTime<Local>, DateTime<Local>)> {
    match self {
      Range::Today => {
        let now = Local::now();
        let start = Local
          .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
          .single()
          .ok_or_else(|| anyhow::anyhow!("Could not create start datetime"))?;

        let end = start + Duration::try_days(1).unwrap();

        Ok((start, end))
      }
      Range::Yesterday => {
        let now = Local::now() - Duration::try_days(1).unwrap();

        let start = Local
          .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
          .single()
          .ok_or_else(|| anyhow::anyhow!("Could not create start datetime"))?;

        let end = start + Duration::try_days(1).unwrap();

        Ok((start, end))
      }
      Range::ThisWeek => {
        let now = Local::now();

        Ok((now.beginning_of_week(), now.end_of_week()))
      }
      Range::LastWeek => {
        let now = Local::now() - Duration::try_weeks(1).unwrap();

        Ok((now.beginning_of_week(), now.end_of_week()))
      }
      Range::ThisMonth => {
        let now = Local::now();

        Ok((now.beginning_of_month(), now.end_of_month()))
      }
      Range::LastMonth => {
        let now = Local::now();

        let date = shift_months(now, -1);

        Ok((date.beginning_of_month(), date.end_of_month()))
      }
      Range::FromTo(start_date, end_date) => {
        let start = start_date.and_hms_opt(0, 0, 0).ok_or_else(|| {
          anyhow::anyhow!(
            "Could not create start datetime from date: {}",
            start_date
          )
        })?;

        let end = end_date.and_hms_opt(0, 0, 0).ok_or_else(|| {
          anyhow::anyhow!(
            "Could not create end datetime from date: {}",
            end_date
          )
        })?;

        let end = end + Duration::try_days(1).unwrap();

        Ok((
          Local.from_local_datetime(&start).unwrap(),
          Local.from_local_datetime(&end).unwrap(),
        ))
      }
      Range::Date(date) => {
        let start = Local
          .with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0)
          .single()
          .ok_or_else(|| {
            anyhow::anyhow!(
              "Could not create start datetime from date: {}",
              date
            )
          })?;

        let end = start + Duration::try_days(1).unwrap();

        Ok((start, end))
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
    if let Ok(range) = self.as_range() {
      let text = format!(
        "{} - {}",
        range.0.format("%Y-%m-%d"),
        range.1.format("%Y-%m-%d")
      );

      write!(f, "{text}")
    } else {
      write!(f, "Invalid range")
    }
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
  pub user: String,
  pub start: DateTime<Utc>,
  pub end: DateTime<Utc>,
  pub dur: u64,
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
