use chrono::{DateTime, Duration, Local};
use chrono::{NaiveDate, Timelike};
use colored::Colorize;
use humantime::format_duration;
use itertools::Itertools;

use crate::{
  client::TogglClient, model::Range, report_client::TogglReportClient,
};

fn formatted_duration(duration: Duration) -> String {
  duration
    .to_std()
    .map_or_else(|_| "".to_string(), |h| format_duration(h).to_string())
}

pub fn detailed(
  debug: bool,
  client: &TogglClient,
  range: &Range,
  report_client: &TogglReportClient,
) -> anyhow::Result<()> {
  let me = client.get_me(debug)?;

  let mut report_details = vec![];

  let (next_row_number, details) =
    report_client.details(debug, me.default_workspace_id, range, None)?;

  for detail in details {
    report_details.push(detail);
  }

  let mut outer_next_row_number = next_row_number;

  while let Some(inner_next_row_number) = outer_next_row_number {
    let (inner_next_row_number, details) = report_client.details(
      debug,
      me.default_workspace_id,
      range,
      Some(inner_next_row_number),
    )?;

    for detail in details {
      report_details.push(detail);
    }

    outer_next_row_number = inner_next_row_number;
  }

  println!("Range: {range}");

  let time_entries_by_user = report_details
    .iter()
    .into_group_map_by(|a| a.username.to_owned());

  if time_entries_by_user.is_empty() {
    println!();
    println!("No time entries found.");

    return Ok(());
  }

  for (user, details) in time_entries_by_user {
    let mut total_seconds = Duration::zero();

    for detail in &details {
      for time_entry in &detail.time_entries {
        total_seconds += Duration::try_seconds(time_entry.seconds as i64)
          .unwrap_or(Duration::zero());
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
      let time_entries = time_entries_by_date.get(date).unwrap();

      let hours = time_entries
        .iter()
        .flat_map(|time_entry| Duration::try_seconds(time_entry.seconds as i64))
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
      let formatted_break = if let Some(r#break) = r#break {
        // between 6 and less than 10 hours, break has to be at least 30 minutes
        if (hours > Duration::try_hours(6).unwrap()
          && hours < Duration::try_hours(10).unwrap())
          && r#break < Duration::try_minutes(30).unwrap()
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
        else if hours > Duration::try_hours(9).unwrap()
          && r#break < Duration::try_minutes(45).unwrap()
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
        "".to_string()
      };

      let formatted_warnings = if !warnings.is_empty() {
        format!(" | {}", warnings.join(", ").bold())
      } else {
        "".to_string()
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
