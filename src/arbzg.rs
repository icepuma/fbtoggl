//! Arbeitszeitgesetz (`ArbZG`) compliance checks.
//!
//! References:
//! - § 2 (Begriffsbestimmungen): `Nachtzeit` = 23:00–06:00 Berlin time.
//!   `Nachtarbeit` = any entry containing more than 2 hours of `Nachtzeit`.
//! - § 3 (Arbeitszeit): daily limit 8h, extendable to 10h only if the
//!   24-week / 6-month average stays at ≤ 8h.
//! - § 4 (Ruhepausen): breaks of ≥ 30 min for work > 6h up to 9h,
//!   ≥ 45 min for work > 9h, in segments of ≥ 15 min; no more than
//!   6 consecutive hours of work without a break.
//! - § 5 (Ruhezeit): ≥ 11 h uninterrupted rest between two workdays.
//! - § 6 (Nacht- und Schichtarbeit): a Nachtarbeitnehmer's daily limit
//!   is 8h, extendable to 10h only if the 4-week / 1-month average
//!   stays at ≤ 8h. We can't compute the averaging here, so we surface
//!   the night-work status as a note.
//!
//! All checks operate in `Europe/Berlin` time (the statute's reference
//! TZ), regardless of where this code runs.

use crate::duration_math::datetime_diff;
use crate::model::ReportTimeEntry;
use chrono::{DateTime, Days, Local, NaiveDate, NaiveTime, TimeZone};
use chrono_tz::Europe::Berlin;
use chrono_tz::Tz;

/// § 4 minimum break-segment length: gaps shorter than this don't reset
/// the consecutive-work counter (i.e. they don't count as a Ruhepause).
pub const MIN_BREAK_SEGMENT_SECS: i64 = 15 * 60;

/// § 3 hard cap (also § 6 cap for Nachtarbeitnehmer).
pub const HARD_CAP_SECS: i64 = 10 * 3600;

/// § 3 / § 6 default daily limit before averaging-Ausgleich kicks in.
pub const SOFT_CAP_SECS: i64 = 8 * 3600;

/// § 4 thresholds.
pub const HOURS_6_SECS: i64 = 6 * 3600;
pub const HOURS_9_SECS: i64 = 9 * 3600;
pub const BREAK_30_MIN_SECS: i64 = 30 * 60;
pub const BREAK_45_MIN_SECS: i64 = 45 * 60;

/// § 4 S.3 cap on consecutive work without a break.
pub const MAX_STREAK_SECS: i64 = 6 * 3600;

/// § 5 minimum rest period between workdays.
pub const MIN_REST_SECS: i64 = 11 * 3600;

/// § 2 (4): an entry is Nachtarbeit if it contains more than 2h of Nachtzeit.
pub const NACHTARBEIT_THRESHOLD_SECS: i64 = 2 * 3600;

/// `Nachtzeit` per § 2 (3) for the general case (not bakeries):
/// 23:00 to 06:00 of the following day.
const NACHTZEIT_START_HOUR: u32 = 23;
const NACHTZEIT_END_HOUR: u32 = 6;

/// Seconds of an entry that fall within `Nachtzeit` (23:00–06:00 Berlin).
pub fn entry_night_seconds(entry: &ReportTimeEntry) -> anyhow::Result<i64> {
  let start_berlin: DateTime<Tz> = entry.start.with_timezone(&Berlin);
  let stop_berlin: DateTime<Tz> = entry.stop.with_timezone(&Berlin);

  if stop_berlin <= start_berlin {
    return Ok(0);
  }

  let mut total: i64 = 0;
  let mut day = start_berlin.date_naive();
  let last_day = stop_berlin.date_naive();

  loop {
    for (h0, h1) in night_windows_for(day)? {
      total = total
        .checked_add(overlap_seconds(start_berlin, stop_berlin, h0, h1))
        .ok_or_else(|| anyhow::anyhow!("night-overlap sum overflow"))?;
    }

    if day >= last_day {
      break;
    }
    day = day
      .checked_add_days(Days::new(1))
      .ok_or_else(|| anyhow::anyhow!("date iteration overflow"))?;
  }

  Ok(total)
}

/// Two Nachtzeit windows that touch the given calendar day in Berlin:
/// `[D 00:00, D 06:00)` and `[D 23:00, (D+1) 00:00)`.
fn night_windows_for(
  day: NaiveDate,
) -> anyhow::Result<[(DateTime<Tz>, DateTime<Tz>); 2]> {
  let midnight = naive_to_berlin(day, 0, 0, 0)?;
  let six_am = naive_to_berlin(day, NACHTZEIT_END_HOUR, 0, 0)?;
  let eleven_pm = naive_to_berlin(day, NACHTZEIT_START_HOUR, 0, 0)?;
  let next_midnight = naive_to_berlin(
    day
      .checked_add_days(Days::new(1))
      .ok_or_else(|| anyhow::anyhow!("date overflow building night window"))?,
    0,
    0,
    0,
  )?;
  Ok([(midnight, six_am), (eleven_pm, next_midnight)])
}

fn naive_to_berlin(
  day: NaiveDate,
  hour: u32,
  minute: u32,
  second: u32,
) -> anyhow::Result<DateTime<Tz>> {
  let naive =
    day.and_time(NaiveTime::from_hms_opt(hour, minute, second).ok_or_else(
      || anyhow::anyhow!("invalid time {hour}:{minute}:{second}"),
    )?);
  Berlin
    .from_local_datetime(&naive)
    .single()
    .ok_or_else(|| anyhow::anyhow!("ambiguous or non-existent Berlin time on {day} {hour}:{minute}:{second} (DST?)"))
}

fn overlap_seconds(
  a_start: DateTime<Tz>,
  a_end: DateTime<Tz>,
  b_start: DateTime<Tz>,
  b_end: DateTime<Tz>,
) -> i64 {
  let s = a_start.max(b_start);
  let e = a_end.min(b_end);
  if e > s {
    datetime_diff(&e, &s).num_seconds().max(0)
  } else {
    0
  }
}

/// Total Nachtzeit seconds across all of a day's entries.
pub fn total_night_seconds(
  entries: &[&ReportTimeEntry],
) -> anyhow::Result<i64> {
  let mut total: i64 = 0;
  for e in entries {
    total = total
      .checked_add(entry_night_seconds(e)?)
      .ok_or_else(|| anyhow::anyhow!("night-seconds overflow"))?;
  }
  Ok(total)
}

/// § 4 S.3: returns `true` if any uninterrupted stretch of work exceeded
/// 6 hours (gaps shorter than 15 minutes don't count as a Ruhepause).
pub fn exceeds_consecutive_work_limit(entries: &[&ReportTimeEntry]) -> bool {
  let mut sorted: Vec<&&ReportTimeEntry> = entries.iter().collect();
  sorted.sort_by_key(|e| e.start);

  let mut streak_start: Option<DateTime<chrono::Utc>> = None;
  let mut prev_stop: Option<DateTime<chrono::Utc>> = None;

  for entry in sorted {
    if let Some(prev) = prev_stop {
      let gap = datetime_diff(&entry.start, &prev).num_seconds();
      if gap >= MIN_BREAK_SEGMENT_SECS || streak_start.is_none() {
        streak_start = Some(entry.start);
      }
    } else {
      streak_start = Some(entry.start);
    }

    if let Some(s) = streak_start {
      let streak = datetime_diff(&entry.stop, &s).num_seconds();
      if streak > MAX_STREAK_SECS {
        return true;
      }
    }

    prev_stop = Some(entry.stop);
  }

  false
}

/// A single day's working-window summary, used for cross-day § 5 checks.
#[derive(Debug, Clone, Copy)]
pub struct DayWindow {
  pub date: NaiveDate,
  pub start: DateTime<Local>,
  pub end: DateTime<Local>,
}

/// § 5: consecutive workdays must be separated by ≥ 11h of rest.
/// Returns the pairs (prev day, next day, observed rest seconds) that
/// violate the rule. Days without entries are skipped (no comparison).
pub fn rest_period_violations(
  days: &[DayWindow],
) -> anyhow::Result<Vec<(DayWindow, DayWindow, i64)>> {
  let mut sorted = days.to_vec();
  sorted.sort_by_key(|d| d.date);

  let mut violations = vec![];
  for pair in sorted.windows(2) {
    let prev = pair
      .first()
      .ok_or_else(|| anyhow::anyhow!("empty window pair"))?;
    let next = pair
      .get(1)
      .ok_or_else(|| anyhow::anyhow!("missing second of window pair"))?;
    let rest = datetime_diff(&next.start, &prev.end).num_seconds();
    if rest < MIN_REST_SECS {
      violations.push((*prev, *next, rest));
    }
  }
  Ok(violations)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "Test code can panic on failure")]
mod tests {
  use super::*;
  use crate::types::TimeEntryId;
  use chrono::Utc;

  fn entry(start: &str, stop: &str) -> ReportTimeEntry {
    let s = DateTime::parse_from_rfc3339(start)
      .unwrap()
      .with_timezone(&Utc);
    let e = DateTime::parse_from_rfc3339(stop)
      .unwrap()
      .with_timezone(&Utc);
    let secs =
      u64::try_from(datetime_diff(&e, &s).num_seconds().max(0)).unwrap();
    ReportTimeEntry {
      id: TimeEntryId::new(1),
      start: s,
      stop: e,
      seconds: secs,
      billable: None,
    }
  }

  fn local_dt(s: &str) -> DateTime<Local> {
    let parsed = DateTime::parse_from_rfc3339(s).unwrap();
    parsed.with_timezone(&Local)
  }

  // Set TZ on test entry so Local maps to Europe/Berlin consistently.
  // (CI/dev machines vary; ctor in client_tests.rs already sets TZ=Europe/Berlin.)

  #[test]
  fn night_seconds_fully_outside_night() {
    // 10:00–12:00 Berlin → 0 night seconds.
    let e = entry("2025-06-10T10:00:00+02:00", "2025-06-10T12:00:00+02:00");
    assert_eq!(entry_night_seconds(&e).unwrap(), 0);
  }

  #[test]
  fn night_seconds_fully_inside_night_early() {
    // 02:00–05:00 Berlin → all 3h are night.
    let e = entry("2025-06-10T02:00:00+02:00", "2025-06-10T05:00:00+02:00");
    assert_eq!(entry_night_seconds(&e).unwrap(), 3 * 3600);
  }

  #[test]
  fn night_seconds_partial_evening() {
    // 22:00–23:30 Berlin → 30 min of Nachtzeit (after 23:00).
    let e = entry("2025-06-10T22:00:00+02:00", "2025-06-10T23:30:00+02:00");
    assert_eq!(entry_night_seconds(&e).unwrap(), 30 * 60);
  }

  #[test]
  fn night_seconds_crossing_midnight() {
    // 22:00 → 01:00 next day = 1h before midnight (23-24) + 1h after.
    let e = entry("2025-06-10T22:00:00+02:00", "2025-06-11T01:00:00+02:00");
    assert_eq!(entry_night_seconds(&e).unwrap(), 2 * 3600);
  }

  #[test]
  fn streak_detects_long_unbroken_run() {
    // 8:00-15:00 with a 5-min gap < 15 min → still one 7h streak.
    let entries = [
      entry("2025-06-10T08:00:00+02:00", "2025-06-10T12:00:00+02:00"),
      entry("2025-06-10T12:05:00+02:00", "2025-06-10T15:00:00+02:00"),
    ];
    let refs: Vec<&ReportTimeEntry> = entries.iter().collect();
    assert!(exceeds_consecutive_work_limit(&refs));
  }

  #[test]
  fn streak_resets_after_real_break() {
    // 30-min break between segments → streak resets, neither half is > 6h.
    let entries = [
      entry("2025-06-10T08:00:00+02:00", "2025-06-10T11:00:00+02:00"),
      entry("2025-06-10T11:30:00+02:00", "2025-06-10T14:30:00+02:00"),
    ];
    let refs: Vec<&ReportTimeEntry> = entries.iter().collect();
    assert!(!exceeds_consecutive_work_limit(&refs));
  }

  #[test]
  fn rest_period_violation_below_eleven_hours() {
    let day1 = DayWindow {
      date: NaiveDate::from_ymd_opt(2025, 6, 10).unwrap(),
      start: local_dt("2025-06-10T09:00:00+02:00"),
      end: local_dt("2025-06-10T22:00:00+02:00"),
    };
    let day2 = DayWindow {
      date: NaiveDate::from_ymd_opt(2025, 6, 11).unwrap(),
      start: local_dt("2025-06-11T07:00:00+02:00"), // only 9h gap
      end: local_dt("2025-06-11T16:00:00+02:00"),
    };
    let violations = rest_period_violations(&[day1, day2]).unwrap();
    assert_eq!(violations.len(), 1);
    let (_, _, rest_secs) = *violations.first().unwrap();
    assert_eq!(rest_secs, 9 * 3600);
  }

  #[test]
  fn rest_period_no_violation_when_eleven_hours_clear() {
    let day1 = DayWindow {
      date: NaiveDate::from_ymd_opt(2025, 6, 10).unwrap(),
      start: local_dt("2025-06-10T09:00:00+02:00"),
      end: local_dt("2025-06-10T20:00:00+02:00"),
    };
    let day2 = DayWindow {
      date: NaiveDate::from_ymd_opt(2025, 6, 11).unwrap(),
      start: local_dt("2025-06-11T07:00:00+02:00"), // 11h gap
      end: local_dt("2025-06-11T16:00:00+02:00"),
    };
    let violations = rest_period_violations(&[day1, day2]).unwrap();
    assert!(violations.is_empty());
  }
}
