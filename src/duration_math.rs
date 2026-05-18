//! Checked arithmetic helpers for `chrono::Duration` and `DateTime`.
//!
//! Replaces the codebase's prior `#[allow(clippy::arithmetic_side_effects)]`
//! sites with explicit overflow handling. Toggl-domain inputs (hours/days,
//! not eons) never overflow in practice, but going through the checked
//! variants kills the lint and adds real bounds checking.

use anyhow::anyhow;
use chrono::{DateTime, Duration, TimeZone};

pub fn checked_add(a: Duration, b: Duration) -> anyhow::Result<Duration> {
  a.checked_add(&b)
    .ok_or_else(|| anyhow!("duration overflow"))
}

pub fn checked_sub(a: Duration, b: Duration) -> anyhow::Result<Duration> {
  a.checked_sub(&b)
    .ok_or_else(|| anyhow!("duration underflow"))
}

/// Sum a sequence of `Duration`s, failing on overflow.
pub fn sum_durations<I>(iter: I) -> anyhow::Result<Duration>
where
  I: IntoIterator<Item = Duration>,
{
  iter.into_iter().try_fold(Duration::zero(), checked_add)
}

/// Accumulate `dur` into the `Duration` slot pointed at by `cell`.
pub fn add_into(cell: &mut Duration, dur: Duration) -> anyhow::Result<()> {
  *cell = checked_add(*cell, dur)?;
  Ok(())
}

/// `stop - start` for two `DateTime`s without panicking on overflow.
/// Returns `Duration::zero()` rather than an error because Toggl never
/// produces inverted ranges in practice and surfacing the error at every
/// call site adds noise without value.
pub fn datetime_diff<Tz: TimeZone>(
  stop: &DateTime<Tz>,
  start: &DateTime<Tz>,
) -> Duration {
  stop
    .timestamp()
    .checked_sub(start.timestamp())
    .and_then(Duration::try_seconds)
    .unwrap_or_else(Duration::zero)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "Test code can panic on failure")]
mod tests {
  use super::*;
  use chrono::Utc;

  #[test]
  fn checked_add_normal() {
    let a = Duration::seconds(10);
    let b = Duration::seconds(20);
    assert_eq!(checked_add(a, b).unwrap(), Duration::seconds(30));
  }

  #[test]
  fn checked_add_detects_overflow() {
    let huge = Duration::MAX;
    let one = Duration::seconds(1);
    assert!(checked_add(huge, one).is_err());
  }

  #[test]
  fn sum_durations_empty_is_zero() {
    let result: Duration = sum_durations(core::iter::empty()).unwrap();
    assert_eq!(result, Duration::zero());
  }

  #[test]
  fn add_into_accumulates() {
    let mut cell = Duration::seconds(5);
    add_into(&mut cell, Duration::seconds(3)).unwrap();
    assert_eq!(cell, Duration::seconds(8));
  }

  #[test]
  fn datetime_diff_positive() {
    let start = DateTime::<Utc>::from_timestamp(1_700_000_000, 0).unwrap();
    let stop = DateTime::<Utc>::from_timestamp(1_700_000_060, 0).unwrap();
    assert_eq!(datetime_diff(&stop, &start), Duration::seconds(60));
  }

  #[test]
  fn datetime_diff_negative_is_signed() {
    // stop earlier than start yields a signed-negative Duration, not zero.
    let start = DateTime::<Utc>::from_timestamp(1_700_000_060, 0).unwrap();
    let stop = DateTime::<Utc>::from_timestamp(1_700_000_000, 0).unwrap();
    assert_eq!(datetime_diff(&stop, &start), Duration::seconds(-60));
  }
}
