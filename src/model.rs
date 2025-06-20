use crate::output::NamedEntity;
use crate::types::{
  ClientId, ProjectId, ProjectStatus, TimeEntryId, WorkspaceId,
};
use chrono::DateTime;
use chrono::Datelike;
use chrono::Duration;
use chrono::Local;
use chrono::NaiveDate;
use chrono::TimeZone;
use chrono::Utc;
use chrono::Weekday;
use chronoutil::shift_months;
use core::fmt;
use core::fmt::Display;
use core::fmt::Formatter;
use core::str::FromStr;
use now::DateTimeNow;
use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Debug)]
pub struct Workspace {
  pub id: WorkspaceId,
  pub name: String,
}

impl NamedEntity for Workspace {
  fn id(&self) -> u64 {
    self.id.0
  }

  fn name(&self) -> &str {
    &self.name
  }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Project {
  pub id: ProjectId,
  pub name: String,
  pub wid: WorkspaceId,
  pub status: ProjectStatus,
  pub cid: Option<ClientId>,
}

impl NamedEntity for Project {
  fn id(&self) -> u64 {
    self.id.0
  }

  fn name(&self) -> &str {
    &self.name
  }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Me {
  pub default_workspace_id: WorkspaceId,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TimeEntry {
  pub id: TimeEntryId,
  pub wid: WorkspaceId,
  pub pid: Option<ProjectId>,
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

#[derive(Deserialize, Debug)]
pub struct TimeEntryDetail {
  pub id: TimeEntryId,
  #[serde(rename = "workspace_id")]
  pub wid: WorkspaceId,
  #[serde(rename = "project_id")]
  pub pid: Option<ProjectId>,
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

impl From<TimeEntryDetail> for TimeEntry {
  fn from(detail: TimeEntryDetail) -> Self {
    Self {
      id: detail.id,
      wid: detail.wid,
      pid: detail.pid,
      billable: detail.billable,
      start: detail.start,
      stop: detail.stop,
      duration: detail.duration,
      description: detail.description,
      tags: detail.tags,
      duronly: detail.duronly,
    }
  }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Client {
  pub id: ClientId,
  pub name: String,
  pub archived: bool,
}

impl NamedEntity for Client {
  fn id(&self) -> u64 {
    self.id.0
  }

  fn name(&self) -> &str {
    &self.name
  }
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
  #[allow(
    clippy::arithmetic_side_effects,
    reason = "Date arithmetic is necessary for iterating through date ranges"
  )]
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

      it += Duration::try_days(1)
        .ok_or_else(|| anyhow::anyhow!("Failed to create duration"))?;
    }

    Ok(missing_days)
  }

  #[allow(
    clippy::arithmetic_side_effects,
    reason = "Date arithmetic is necessary for calculating date ranges"
  )]
  pub fn as_range(self) -> anyhow::Result<(DateTime<Local>, DateTime<Local>)> {
    match self {
      Self::Today => {
        let now = Local::now();
        let start = Local
          .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
          .single()
          .ok_or_else(|| anyhow::anyhow!("Could not create start datetime"))?;

        let end = start
          + Duration::try_days(1)
            .ok_or_else(|| anyhow::anyhow!("Failed to create duration"))?;

        Ok((start, end))
      }
      Self::Yesterday => {
        let now = Local::now()
          - Duration::try_days(1)
            .ok_or_else(|| anyhow::anyhow!("Failed to create duration"))?;

        let start = Local
          .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
          .single()
          .ok_or_else(|| anyhow::anyhow!("Could not create start datetime"))?;

        let end = start
          + Duration::try_days(1)
            .ok_or_else(|| anyhow::anyhow!("Failed to create duration"))?;

        Ok((start, end))
      }
      Self::ThisWeek => {
        let now = Local::now();

        Ok((now.beginning_of_week(), now.end_of_week()))
      }
      Self::LastWeek => {
        let now = Local::now()
          - Duration::try_weeks(1)
            .ok_or_else(|| anyhow::anyhow!("Failed to create duration"))?;

        Ok((now.beginning_of_week(), now.end_of_week()))
      }
      Self::ThisMonth => {
        let now = Local::now();

        Ok((now.beginning_of_month(), now.end_of_month()))
      }
      Self::LastMonth => {
        let now = Local::now();

        let date = shift_months(now, -1);

        Ok((date.beginning_of_month(), date.end_of_month()))
      }
      Self::FromTo(start_date, end_date) => {
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

        let end = end
          + Duration::try_days(1).ok_or_else(|| {
            anyhow::anyhow!("Failed to add one day to end date")
          })?;

        Ok((
          Local.from_local_datetime(&start).single().ok_or_else(|| {
            anyhow::anyhow!("Could not convert start to local datetime")
          })?,
          Local.from_local_datetime(&end).single().ok_or_else(|| {
            anyhow::anyhow!("Could not convert end to local datetime")
          })?,
        ))
      }
      Self::Date(date) => {
        let start = Local
          .with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0)
          .single()
          .ok_or_else(|| {
            anyhow::anyhow!(
              "Could not create start datetime from date: {}",
              date
            )
          })?;

        let end = start
          + Duration::try_days(1)
            .ok_or_else(|| anyhow::anyhow!("Failed to add one day to date"))?;

        Ok((start, end))
      }
    }
  }
}

impl FromStr for Range {
  type Err = anyhow::Error;

  #[allow(
    clippy::arithmetic_side_effects,
    reason = "String slicing with known delimiter position is safe"
  )]
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "today" => Ok(Self::Today),
      "yesterday" => Ok(Self::Yesterday),
      "this-week" => Ok(Self::ThisWeek),
      "last-week" => Ok(Self::LastWeek),
      "this-month" => Ok(Self::ThisMonth),
      "last-month" => Ok(Self::LastMonth),
      from_to_or_date => match from_to_or_date.find('|') {
        Some(index) => {
          let start =
            NaiveDate::parse_from_str(&from_to_or_date[..index], "%Y-%m-%d")
              .map_err(|e| anyhow::anyhow!("Invalid start date: {}", e))?;
          let end = NaiveDate::parse_from_str(
            &from_to_or_date[index + 1..],
            "%Y-%m-%d",
          )
          .map_err(|e| anyhow::anyhow!("Invalid end date: {}", e))?;

          if start > end {
            return Err(anyhow::anyhow!(
              "Start date must be before or equal to end date"
            ));
          }

          Ok(Self::FromTo(start, end))
        }
        None => Ok(Self::Date(from_to_or_date.parse().map_err(|_| {
          anyhow::anyhow!("Invalid date format. Expected YYYY-MM-DD")
        })?)),
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
  pub id: TimeEntryId,
  pub start: DateTime<Utc>,
  pub stop: DateTime<Utc>,
  pub seconds: u64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ReportDetails {
  pub username: String,
  pub time_entries: Vec<ReportTimeEntry>,
}
