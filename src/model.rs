use crate::output::NamedEntity;
use crate::types::{
  ClientId, ProjectId, ProjectStatus, TimeEntryId, WorkspaceId,
};
use chrono::{
  DateTime, Datelike, Days, Local, NaiveDate, TimeZone, Utc, Weekday,
};
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
    self.id.get()
  }

  fn name(&self) -> &str {
    &self.name
  }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Project {
  pub id: ProjectId,
  pub name: String,
  pub workspace_id: WorkspaceId,
  pub status: Option<ProjectStatus>,
  pub active: Option<bool>,
  pub client_id: Option<ClientId>,
}

impl NamedEntity for Project {
  fn id(&self) -> u64 {
    self.id.get()
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
  pub workspace_id: WorkspaceId,
  pub project_id: Option<ProjectId>,
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
  pub id: ClientId,
  pub name: String,
  pub archived: bool,
}

impl NamedEntity for Client {
  fn id(&self) -> u64 {
    self.id.get()
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
  pub fn get_datetimes(self) -> anyhow::Result<Vec<DateTime<Local>>> {
    self.get_datetimes_from(Local::now())
  }

  fn get_datetimes_from(
    self,
    now: DateTime<Local>,
  ) -> anyhow::Result<Vec<DateTime<Local>>> {
    let (start, end) = self.as_dates_from(now)?;
    let mut it = start;
    let mut missing_days = vec![];

    while it <= end {
      let weekday = it.weekday();

      if weekday != Weekday::Sat && weekday != Weekday::Sun {
        missing_days.push(local_midnight(it)?);
      }

      it = it
        .checked_add_days(Days::new(1))
        .ok_or_else(|| anyhow::anyhow!("Date iteration overflowed"))?;
    }

    Ok(missing_days)
  }

  pub fn as_dates(self) -> anyhow::Result<(NaiveDate, NaiveDate)> {
    self.as_dates_from(Local::now())
  }

  fn as_dates_from(
    self,
    now: DateTime<Local>,
  ) -> anyhow::Result<(NaiveDate, NaiveDate)> {
    match self {
      Self::Today => {
        let today = now.date_naive();

        Ok((today, today))
      }
      Self::Yesterday => {
        let yesterday = now
          .date_naive()
          .checked_sub_days(Days::new(1))
          .ok_or_else(|| anyhow::anyhow!("Date arithmetic overflowed"))?;

        Ok((yesterday, yesterday))
      }
      Self::ThisWeek => Ok((
        now.beginning_of_week().date_naive(),
        now.end_of_week().date_naive(),
      )),
      Self::LastWeek => {
        let now = now
          .checked_sub_days(Days::new(7))
          .ok_or_else(|| anyhow::anyhow!("Date arithmetic overflowed"))?;

        Ok((
          now.beginning_of_week().date_naive(),
          now.end_of_week().date_naive(),
        ))
      }
      Self::ThisMonth => Ok((
        now.beginning_of_month().date_naive(),
        now.end_of_month().date_naive(),
      )),
      Self::LastMonth => {
        let date = shift_months(now, -1);

        Ok((
          date.beginning_of_month().date_naive(),
          date.end_of_month().date_naive(),
        ))
      }
      Self::FromTo(start_date, end_date) => Ok((start_date, end_date)),
      Self::Date(date) => Ok((date, date)),
    }
  }

  pub fn as_time_entries_dates(self) -> anyhow::Result<(NaiveDate, NaiveDate)> {
    self.as_time_entries_dates_from(Local::now())
  }

  fn as_time_entries_dates_from(
    self,
    now: DateTime<Local>,
  ) -> anyhow::Result<(NaiveDate, NaiveDate)> {
    let (start, end) = self.as_dates_from(now)?;
    let exclusive_end = end
      .checked_add_days(Days::new(1))
      .ok_or_else(|| anyhow::anyhow!("Date arithmetic overflowed"))?;

    Ok((start, exclusive_end))
  }
}

fn local_midnight(date: NaiveDate) -> anyhow::Result<DateTime<Local>> {
  Local
    .with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0)
    .single()
    .ok_or_else(|| {
      anyhow::anyhow!("Could not create start datetime from date: {date}")
    })
}

impl FromStr for Range {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "today" => Ok(Self::Today),
      "yesterday" => Ok(Self::Yesterday),
      "this-week" => Ok(Self::ThisWeek),
      "last-week" => Ok(Self::LastWeek),
      "this-month" => Ok(Self::ThisMonth),
      "last-month" => Ok(Self::LastMonth),
      from_to_or_date => match from_to_or_date.split_once('|') {
        Some((start_str, end_str)) => {
          let start = NaiveDate::parse_from_str(start_str, "%Y-%m-%d")
            .map_err(|e| anyhow::anyhow!("Invalid start date: {e}"))?;
          let end = NaiveDate::parse_from_str(end_str, "%Y-%m-%d")
            .map_err(|e| anyhow::anyhow!("Invalid end date: {e}"))?;

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
    if let Ok(range) = self.as_dates() {
      write!(
        f,
        "{} - {}",
        range.0.format("%Y-%m-%d"),
        range.1.format("%Y-%m-%d")
      )
    } else {
      write!(f, "Invalid range")
    }
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used, reason = "Test code can panic on failure")]

  use super::Range;
  use chrono::{DateTime, Local, NaiveDate, TimeZone};
  use pretty_assertions::assert_eq;

  fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
  }

  fn fixed_now() -> DateTime<Local> {
    Local
      .with_ymd_and_hms(2024, 3, 15, 12, 0, 0)
      .single()
      .unwrap()
  }

  #[test]
  fn date_range_display_uses_inclusive_end_date() {
    let date = date(2021, 11, 21);

    assert_eq!(Range::Date(date).to_string(), "2021-11-21 - 2021-11-21");
  }

  #[test]
  fn preset_ranges_are_inclusive_calendar_dates() -> anyhow::Result<()> {
    let cases = [
      (Range::Today, (date(2024, 3, 15), date(2024, 3, 15))),
      (Range::Yesterday, (date(2024, 3, 14), date(2024, 3, 14))),
      (Range::ThisWeek, (date(2024, 3, 11), date(2024, 3, 17))),
      (Range::LastWeek, (date(2024, 3, 4), date(2024, 3, 10))),
      (Range::ThisMonth, (date(2024, 3, 1), date(2024, 3, 31))),
      (Range::LastMonth, (date(2024, 2, 1), date(2024, 2, 29))),
      (
        Range::Date(date(2024, 3, 2)),
        (date(2024, 3, 2), date(2024, 3, 2)),
      ),
      (
        Range::FromTo(date(2024, 3, 1), date(2024, 3, 2)),
        (date(2024, 3, 1), date(2024, 3, 2)),
      ),
    ];

    for (range, expected) in cases {
      assert_eq!(range.as_dates_from(fixed_now())?, expected, "{range:?}");
    }

    Ok(())
  }

  #[test]
  fn time_entry_ranges_use_exclusive_end_date() -> anyhow::Result<()> {
    let cases = [
      (Range::Today, (date(2024, 3, 15), date(2024, 3, 16))),
      (Range::Yesterday, (date(2024, 3, 14), date(2024, 3, 15))),
      (Range::ThisWeek, (date(2024, 3, 11), date(2024, 3, 18))),
      (Range::LastWeek, (date(2024, 3, 4), date(2024, 3, 11))),
      (Range::ThisMonth, (date(2024, 3, 1), date(2024, 4, 1))),
      (Range::LastMonth, (date(2024, 2, 1), date(2024, 3, 1))),
      (
        Range::Date(date(2024, 3, 2)),
        (date(2024, 3, 2), date(2024, 3, 3)),
      ),
      (
        Range::FromTo(date(2024, 3, 1), date(2024, 3, 2)),
        (date(2024, 3, 1), date(2024, 3, 3)),
      ),
    ];

    for (range, expected) in cases {
      assert_eq!(
        range.as_time_entries_dates_from(fixed_now())?,
        expected,
        "{range:?}"
      );
    }

    Ok(())
  }

  #[test]
  fn this_week_missing_dates_skip_weekend() -> anyhow::Result<()> {
    let days = Range::ThisWeek
      .get_datetimes_from(fixed_now())?
      .into_iter()
      .map(|dt| dt.date_naive())
      .collect::<Vec<_>>();

    assert_eq!(
      days,
      vec![
        date(2024, 3, 11),
        date(2024, 3, 12),
        date(2024, 3, 13),
        date(2024, 3, 14),
        date(2024, 3, 15),
      ]
    );

    Ok(())
  }

  #[test]
  fn from_to_get_datetimes_includes_end_date() -> anyhow::Result<()> {
    let days = Range::FromTo(date(2021, 11, 1), date(2021, 11, 3))
      .get_datetimes()?
      .into_iter()
      .map(|dt| dt.date_naive())
      .collect::<Vec<_>>();

    assert_eq!(
      days,
      vec![date(2021, 11, 1), date(2021, 11, 2), date(2021, 11, 3),]
    );

    Ok(())
  }

  #[test]
  fn yesterday_is_single_calendar_day() -> anyhow::Result<()> {
    let expected = date(2024, 3, 14);

    let (start, end) = Range::Yesterday.as_dates_from(fixed_now())?;

    assert_eq!(start, expected);
    assert_eq!(end, expected);
    assert_eq!(Range::Yesterday.get_datetimes_from(fixed_now())?.len(), 1);

    Ok(())
  }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ReportTimeEntry {
  pub id: TimeEntryId,
  pub start: DateTime<Utc>,
  pub stop: DateTime<Utc>,
  pub seconds: u64,
  #[serde(default)]
  pub billable: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ReportDetails {
  pub username: String,
  pub time_entries: Vec<ReportTimeEntry>,
}
