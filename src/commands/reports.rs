use chrono::{DateTime, Duration, Local};
use chrono::{NaiveDate, Timelike};
use colored::Colorize;
use humantime::format_duration;
use itertools::Itertools;

use crate::{
  cli::Format, client::TogglClient, model::Range,
  report_client::TogglReportClient,
};

// Duration constants to avoid unwrap calls
const HOURS_6: i64 = 6 * 3600;
const HOURS_9: i64 = 9 * 3600;
const MINUTES_30: i64 = 30 * 60;
const MINUTES_45: i64 = 45 * 60;

fn formatted_duration(duration: Duration) -> String {
  duration
    .to_std()
    .map_or_else(|_| String::new(), |h| format_duration(h).to_string())
}

#[allow(
  clippy::too_many_lines,
  reason = "Main function coordinating report generation - splitting would reduce readability"
)]
#[allow(
  clippy::arithmetic_side_effects,
  reason = "Duration arithmetic is necessary throughout this function for time calculations"
)]
pub fn detailed(
  debug: bool,
  client: &TogglClient,
  range: &Range,
  report_client: &TogglReportClient,
) -> anyhow::Result<()> {
  let me = client.get_me(debug)?;

  let report_details =
    report_client.detailed(me.default_workspace_id, range, debug)?;

  println!("Range: {range}");

  let time_entries_by_user =
    report_details.iter().into_group_map_by(|a| &a.username);

  if time_entries_by_user.is_empty() {
    println!();
    println!("No time entries found.");

    return Ok(());
  }

  for (user, details) in time_entries_by_user {
    let mut total_seconds = Duration::zero();

    for detail in &details {
      for time_entry in &detail.time_entries {
        if let Ok(seconds) = i64::try_from(time_entry.seconds) {
          if let Some(duration) = Duration::try_seconds(seconds) {
            total_seconds += duration;
          }
        }
      }
    }

    println!();
    println!(
      "{} - {} hours ({})",
      user,
      total_seconds.num_hours(),
      formatted_duration(total_seconds)
    );
    println!();

    let mut time_entries = vec![];

    for detail in &details {
      for time_entry in &detail.time_entries {
        time_entries.push(time_entry);
      }
    }

    let time_entries_by_date = time_entries
      .iter()
      .into_group_map_by(|time_entry| time_entry.start.date_naive());

    let mut dates = time_entries_by_date.keys().collect::<Vec<&NaiveDate>>();
    dates.sort();

    for date in dates {
      let time_entries = time_entries_by_date
        .get(date)
        .ok_or_else(|| anyhow::anyhow!("Missing time entries for date"))?;

      let hours = time_entries
        .iter()
        .filter_map(|time_entry| {
          i64::try_from(time_entry.seconds)
            .ok()
            .and_then(Duration::try_seconds)
        })
        .fold(Duration::zero(), |a, b| a + b);

      let start = time_entries
        .iter()
        .min_by_key(|time_entry| time_entry.start)
        .map(|time_entry| DateTime::<Local>::from(time_entry.start));

      let end = time_entries
        .iter()
        .max_by_key(|time_entry| time_entry.stop)
        .map(|time_entry| DateTime::<Local>::from(time_entry.stop));

      let r#break = if let (Some(start), Some(end)) = (start, end) {
        let total = end - start;

        Some(total - hours)
      } else {
        None
      };

      let mut warnings = vec![];

      if hours.num_hours() > 10 {
        warnings.push("More than 10 hours".red().to_string());
      }

      if let Some(start) = start {
        if start.time().hour() < 6 {
          warnings.push("Start time is before 6am".red().to_string());
        }
      }

      if let Some(end) = end {
        if end.time().hour() > 22 {
          warnings.push("End time is after 10pm".red().to_string());
        }
      }

      let hours_formatted = formatted_duration(hours);

      // https://www.gesetze-im-internet.de/arbzg/__4.html#:~:text=Arbeitszeitgesetz%20(ArbZG),neun%20Stunden%20insgesamt%20zu%20unterbrechen.
      #[allow(
        clippy::option_if_let_else,
        reason = "Complex if-let with multiple conditions is more readable than map_or_else"
      )]
      let formatted_break = if let Some(r#break) = r#break {
        // between 6 and up to 9 hours, break has to be at least 30 minutes
        if (hours.num_seconds() > HOURS_6 && hours.num_seconds() <= HOURS_9)
          && r#break.num_seconds() < MINUTES_30
        {
          warnings.push(
              format!(
                "Worked for {hours_formatted} => break should be at least 30 minutes!"
              )
              .red()
              .to_string(),
            );
        }
        // more than 9 hours, break has to be at least 45 minutes
        else if hours.num_seconds() > HOURS_9
          && r#break.num_seconds() < MINUTES_45
        {
          warnings.push(
              format!(
                "Worked for {hours_formatted} => break should be at least 45 minutes!"
              )
              .red()
              .to_string(),
            );
        }

        format!(", Break: {}", formatted_duration(r#break))
      } else {
        String::new()
      };

      let formatted_warnings = if warnings.is_empty() {
        String::new()
      } else {
        format!(" | {}", warnings.join(", ").bold())
      };

      println!(
        "{} - {} - {} | Work: {}{}{}",
        date.format("%Y-%m-%d"),
        start
          .map(|s| s.format("%H:%M").to_string())
          .unwrap_or_default(),
        end
          .map(|s| s.format("%H:%M").to_string())
          .unwrap_or_default(),
        hours_formatted,
        formatted_break,
        formatted_warnings
      );
    }
  }

  Ok(())
}

#[allow(
  clippy::arithmetic_side_effects,
  reason = "Duration arithmetic is necessary and safe in this context"
)]
#[allow(
  clippy::cast_precision_loss,
  clippy::as_conversions,
  reason = "Converting to f64 for percentage calculations is acceptable here"
)]
pub fn summary(
  debug: bool,
  client: &TogglClient,
  range: &Range,
  _format: &Format,
) -> anyhow::Result<()> {
  let time_entries = client.get_time_entries(debug, range)?;

  // Calculate summary statistics
  let total_duration: Duration = time_entries
    .iter()
    .filter_map(|e| e.stop.map(|stop| stop - e.start))
    .sum();

  let billable_duration: Duration = time_entries
    .iter()
    .filter(|e| e.billable.unwrap_or(false))
    .filter_map(|e| e.stop.map(|stop| stop - e.start))
    .sum();

  let non_billable_duration = total_duration - billable_duration;

  let entries_count = time_entries.len();
  let billable_count = time_entries
    .iter()
    .filter(|e| e.billable.unwrap_or(false))
    .count();

  // Group by project
  let mut project_durations = std::collections::HashMap::new();
  for entry in &time_entries {
    if let Some(project_id) = entry.pid {
      let duration = entry
        .stop
        .map(|stop| stop - entry.start)
        .unwrap_or_default();
      *project_durations
        .entry(project_id)
        .or_insert(Duration::zero()) += duration;
    }
  }

  println!("Summary for {range}");
  println!();
  println!("Total time: {}", formatted_duration(total_duration));
  println!(
    "Billable: {} ({:.1}%)",
    formatted_duration(billable_duration),
    (billable_duration.num_seconds() as f64
      / total_duration.num_seconds() as f64)
      * 100.0
  );
  println!(
    "Non-billable: {} ({:.1}%)",
    formatted_duration(non_billable_duration),
    (non_billable_duration.num_seconds() as f64
      / total_duration.num_seconds() as f64)
      * 100.0
  );
  println!();
  println!("Total entries: {entries_count}");
  println!(
    "Billable entries: {} ({:.1}%)",
    billable_count,
    (billable_count as f64 / entries_count as f64) * 100.0
  );

  Ok(())
}
