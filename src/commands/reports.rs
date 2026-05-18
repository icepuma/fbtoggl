use chrono::{DateTime, Duration, Local, NaiveDate};
use colored::Colorize;
use humantime::format_duration;
use itertools::Itertools;

use crate::{
  arbzg::{
    self, BREAK_30_MIN_SECS, BREAK_45_MIN_SECS, DayWindow, HARD_CAP_SECS,
    HOURS_6_SECS, HOURS_9_SECS, NACHTARBEIT_THRESHOLD_SECS, SOFT_CAP_SECS,
  },
  cli::GroupBy,
  client::TogglClient,
  duration_math::{add_into, checked_sub, datetime_diff, sum_durations},
  model::Range,
  report_client::TogglReportClient,
};

fn formatted_duration(duration: Duration) -> String {
  duration
    .to_std()
    .map_or_else(|_| String::new(), |h| format_duration(h).to_string())
}

pub fn detailed(
  client: &TogglClient,
  range: &Range,
  billable_only: bool,
  compliance: bool,
  report_client: &TogglReportClient,
) -> anyhow::Result<()> {
  let me = client.get_me()?;

  let report_details =
    report_client.detailed(me.default_workspace_id, range)?;

  println!("Range: {range}");

  let time_entries_by_user =
    report_details.iter().into_group_map_by(|a| &a.username);

  if time_entries_by_user.is_empty() {
    println!();
    println!("No time entries found.");

    return Ok(());
  }

  for (user, details) in time_entries_by_user {
    let time_entries: Vec<_> = details
      .iter()
      .flat_map(|d| &d.time_entries)
      .filter(|te| !billable_only || te.billable.unwrap_or(false))
      .collect();

    let total_seconds = sum_durations(time_entries.iter().filter_map(|te| {
      i64::try_from(te.seconds)
        .ok()
        .and_then(Duration::try_seconds)
    }))?;

    println!();
    println!(
      "{} - {} hours ({})",
      user,
      total_seconds.num_hours(),
      formatted_duration(total_seconds)
    );
    println!();

    let time_entries_by_date = time_entries
      .iter()
      .into_group_map_by(|time_entry| time_entry.start.date_naive());

    let mut dates = time_entries_by_date.keys().collect::<Vec<&NaiveDate>>();
    dates.sort();

    let mut day_windows: Vec<DayWindow> = vec![];

    for date in dates {
      let time_entries = time_entries_by_date
        .get(date)
        .ok_or_else(|| anyhow::anyhow!("Missing time entries for date"))?;
      if let Some(window) = render_day_line(*date, time_entries, compliance)? {
        day_windows.push(window);
      }
    }

    if compliance {
      print_section_5_violations(&day_windows)?;
    }
  }

  Ok(())
}

fn render_day_line(
  date: NaiveDate,
  time_entries: &[&&crate::model::ReportTimeEntry],
  compliance: bool,
) -> anyhow::Result<Option<DayWindow>> {
  let hours = sum_durations(time_entries.iter().filter_map(|te| {
    i64::try_from(te.seconds)
      .ok()
      .and_then(Duration::try_seconds)
  }))?;

  let start = time_entries
    .iter()
    .min_by_key(|te| te.start)
    .map(|te| DateTime::<Local>::from(te.start));

  let end = time_entries
    .iter()
    .max_by_key(|te| te.stop)
    .map(|te| DateTime::<Local>::from(te.stop));

  let r#break = match (start, end) {
    (Some(s), Some(e)) => Some(checked_sub(datetime_diff(&e, &s), hours)?),
    _ => None,
  };

  let mut warnings: Vec<String> = vec![];

  if compliance {
    collect_arbzg_warnings(hours, r#break, time_entries, &mut warnings)?;
  }

  let hours_formatted = formatted_duration(hours);
  let formatted_break = r#break
    .map(|b| format!(", Break: {}", formatted_duration(b)))
    .unwrap_or_default();
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

  Ok(match (start, end) {
    (Some(start), Some(end)) => Some(DayWindow { date, start, end }),
    _ => None,
  })
}

fn collect_arbzg_warnings(
  hours: Duration,
  r#break: Option<Duration>,
  time_entries: &[&&crate::model::ReportTimeEntry],
  warnings: &mut Vec<String>,
) -> anyhow::Result<()> {
  let hours_secs = hours.num_seconds();
  let hours_formatted = formatted_duration(hours);

  // Flatten &&ReportTimeEntry to &ReportTimeEntry so the arbzg helpers
  // don't need to know about the report code's HashMap value shape.
  let flat: Vec<&crate::model::ReportTimeEntry> =
    time_entries.iter().map(|e| **e).collect();

  // § 2 (3) + § 6: detect Nachtarbeit (more than 2h Nachtzeit in a day).
  let night_secs = arbzg::total_night_seconds(&flat)?;
  let is_night_work = night_secs > NACHTARBEIT_THRESHOLD_SECS;
  if is_night_work {
    warnings.push(
      format!(
        "ArbZG §2(4)/§6: night work ({} of Nachtzeit) — 8h cap applies, \
         Ausgleich required within 4 weeks / 1 month",
        formatted_duration(
          Duration::try_seconds(night_secs).unwrap_or_default()
        )
      )
      .yellow()
      .to_string(),
    );
  }

  // § 3 (and § 6 for Nachtarbeitnehmer): hard daily cap is 10h.
  if hours_secs > HARD_CAP_SECS {
    warnings.push(
      format!("ArbZG §3: {hours_formatted} exceeds the 10h hard daily limit")
        .red()
        .to_string(),
    );
  } else if hours_secs > SOFT_CAP_SECS {
    // > 8h is only permitted with averaging-Ausgleich.
    let window = if is_night_work {
      "4 weeks / 1 month (§ 6)"
    } else {
      "24 weeks / 6 months (§ 3)"
    };
    warnings.push(
      format!(
        "ArbZG: {hours_formatted} > 8h — verify Ausgleich to 8h avg over {window}"
      )
      .yellow()
      .to_string(),
    );
  }

  // § 4 S.1: break thresholds.
  if let Some(b) = r#break {
    let break_secs = b.num_seconds();
    if (hours_secs > HOURS_6_SECS && hours_secs <= HOURS_9_SECS)
      && break_secs < BREAK_30_MIN_SECS
    {
      warnings.push(
        format!(
          "ArbZG §4: {hours_formatted} requires ≥ 30 min break (got {})",
          formatted_duration(b)
        )
        .red()
        .to_string(),
      );
    } else if hours_secs > HOURS_9_SECS && break_secs < BREAK_45_MIN_SECS {
      warnings.push(
        format!(
          "ArbZG §4: {hours_formatted} requires ≥ 45 min break (got {})",
          formatted_duration(b)
        )
        .red()
        .to_string(),
      );
    }
  }

  // § 4 S.3: no more than 6 consecutive hours of work without a break
  // (gaps shorter than 15 min don't count as a Ruhepause).
  if arbzg::exceeds_consecutive_work_limit(&flat) {
    warnings.push(
      "ArbZG §4 S.3: more than 6h worked without a ≥ 15 min break"
        .red()
        .to_string(),
    );
  }

  Ok(())
}

/// § 5: print any < 11h gaps between consecutive workdays for this user.
fn print_section_5_violations(days: &[DayWindow]) -> anyhow::Result<()> {
  let violations = arbzg::rest_period_violations(days)?;
  for (prev, next, rest_secs) in violations {
    let rest = Duration::try_seconds(rest_secs).unwrap_or_default();
    println!(
      "  {}",
      format!(
        "ArbZG §5: only {} rest between {} ({}) and {} ({}) — needs ≥ 11h",
        formatted_duration(rest),
        prev.date.format("%Y-%m-%d"),
        prev.end.format("%H:%M"),
        next.date.format("%Y-%m-%d"),
        next.start.format("%H:%M"),
      )
      .red()
    );
  }
  Ok(())
}

#[allow(
  clippy::cast_precision_loss,
  clippy::as_conversions,
  reason = "f64 percentages — entry counts won't exceed 2^53"
)]
pub fn summary(
  client: &TogglClient,
  range: &Range,
  group_by: Option<GroupBy>,
  billable_only: bool,
) -> anyhow::Result<()> {
  let raw_entries = client.get_time_entries(range)?;

  let time_entries: Vec<_> = raw_entries
    .into_iter()
    .filter(|e| !billable_only || e.billable.unwrap_or(false))
    .collect();

  let total_duration = sum_durations(
    time_entries
      .iter()
      .filter_map(|e| e.stop.map(|stop| datetime_diff(&stop, &e.start))),
  )?;

  let billable_duration = sum_durations(
    time_entries
      .iter()
      .filter(|e| e.billable.unwrap_or(false))
      .filter_map(|e| e.stop.map(|stop| datetime_diff(&stop, &e.start))),
  )?;

  let non_billable_duration = checked_sub(total_duration, billable_duration)?;

  let entries_count = time_entries.len();
  let billable_count = time_entries
    .iter()
    .filter(|e| e.billable.unwrap_or(false))
    .count();

  println!(
    "Summary for {range}{}",
    if billable_only {
      " (billable only)"
    } else {
      ""
    }
  );
  println!();
  println!("Total time: {}", formatted_duration(total_duration));

  let total_seconds = total_duration.num_seconds() as f64;
  let billable_pct = if total_seconds > 0.0 {
    (billable_duration.num_seconds() as f64 / total_seconds) * 100.0
  } else {
    0.0
  };
  let non_billable_pct = if total_seconds > 0.0 {
    (non_billable_duration.num_seconds() as f64 / total_seconds) * 100.0
  } else {
    0.0
  };

  println!(
    "Billable: {} ({:.1}%)",
    formatted_duration(billable_duration),
    billable_pct
  );
  println!(
    "Non-billable: {} ({:.1}%)",
    formatted_duration(non_billable_duration),
    non_billable_pct
  );
  println!();
  println!("Total entries: {entries_count}");

  let entries_pct = if entries_count > 0 {
    (billable_count as f64 / entries_count as f64) * 100.0
  } else {
    0.0
  };
  println!("Billable entries: {billable_count} ({entries_pct:.1}%)");

  if let Some(dim) = group_by {
    println!();
    render_group_by(client, &time_entries, dim, total_duration)?;
  }

  Ok(())
}

#[allow(
  clippy::cast_precision_loss,
  clippy::as_conversions,
  reason = "f64 percentages — entry counts won't exceed 2^53"
)]
fn render_group_by(
  client: &TogglClient,
  entries: &[crate::model::TimeEntry],
  dim: GroupBy,
  total: Duration,
) -> anyhow::Result<()> {
  let mut buckets = match dim {
    GroupBy::Project => bucket_by_project(client, entries)?,
    GroupBy::Client => bucket_by_client(client, entries)?,
    GroupBy::Tag => bucket_by_tag(entries)?,
    GroupBy::Day => bucket_by_day(entries)?,
  };

  let label = match dim {
    GroupBy::Project => "project",
    GroupBy::Client => "client",
    GroupBy::Tag => "tag",
    GroupBy::Day => "day",
  };

  if buckets.is_empty() {
    println!("By {label}: (no entries)");
    return Ok(());
  }

  // Day buckets sort ascending (YYYY-MM-DD strings); everything else
  // descends by duration so the biggest bucket is on top.
  if matches!(dim, GroupBy::Day) {
    buckets.sort_by(|a, b| a.0.cmp(&b.0));
  } else {
    buckets.sort_by_key(|(_, dur)| core::cmp::Reverse(*dur));
  }

  println!("By {label}:");
  let total_seconds = total.num_seconds() as f64;
  for (name, dur) in &buckets {
    let pct = if total_seconds > 0.0 {
      (dur.num_seconds() as f64 / total_seconds) * 100.0
    } else {
      0.0
    };
    println!("  {} - {} ({:.1}%)", name, formatted_duration(*dur), pct);
  }

  Ok(())
}

fn entry_duration(e: &crate::model::TimeEntry) -> Duration {
  e.stop
    .map(|s| datetime_diff(&s, &e.start))
    .unwrap_or_default()
}

fn bucket_by_project(
  client: &TogglClient,
  entries: &[crate::model::TimeEntry],
) -> anyhow::Result<Vec<(String, Duration)>> {
  use std::collections::HashMap;

  let me = client.get_me()?;
  let projects =
    client.get_workspace_projects(true, me.default_workspace_id)?;
  let project_name: HashMap<_, _> =
    projects.iter().map(|p| (p.id, p.name.as_str())).collect();

  let mut totals: HashMap<String, Duration> = HashMap::new();
  for e in entries {
    let key = e
      .project_id
      .and_then(|pid| project_name.get(&pid).copied())
      .unwrap_or("(no project)")
      .to_owned();
    add_into(
      totals.entry(key).or_insert(Duration::zero()),
      entry_duration(e),
    )?;
  }
  Ok(totals.into_iter().collect())
}

fn bucket_by_client(
  client: &TogglClient,
  entries: &[crate::model::TimeEntry],
) -> anyhow::Result<Vec<(String, Duration)>> {
  use std::collections::HashMap;

  let me = client.get_me()?;
  let workspace_id = me.default_workspace_id;
  let projects = client.get_workspace_projects(true, workspace_id)?;
  let clients = client
    .get_workspace_clients(true, workspace_id)?
    .unwrap_or_default();

  let client_name: HashMap<_, _> =
    clients.iter().map(|c| (c.id, c.name.as_str())).collect();
  let project_client: HashMap<_, _> =
    projects.iter().map(|p| (p.id, p.client_id)).collect();

  let mut totals: HashMap<String, Duration> = HashMap::new();
  for e in entries {
    let key = e
      .project_id
      .and_then(|pid| project_client.get(&pid).copied().flatten())
      .and_then(|cid| client_name.get(&cid).copied())
      .unwrap_or("(no client)")
      .to_owned();
    add_into(
      totals.entry(key).or_insert(Duration::zero()),
      entry_duration(e),
    )?;
  }
  Ok(totals.into_iter().collect())
}

fn bucket_by_tag(
  entries: &[crate::model::TimeEntry],
) -> anyhow::Result<Vec<(String, Duration)>> {
  use std::collections::HashMap;

  // Untagged entries land in "(no tag)"; tagged entries contribute to every
  // one of their tags, so summed tag totals can exceed the grand total.
  let mut totals: HashMap<String, Duration> = HashMap::new();
  for e in entries {
    let dur = entry_duration(e);
    match e.tags.as_ref().filter(|t| !t.is_empty()) {
      Some(tags) => {
        for tag in tags {
          add_into(totals.entry(tag.clone()).or_insert(Duration::zero()), dur)?;
        }
      }
      None => {
        add_into(
          totals
            .entry("(no tag)".to_owned())
            .or_insert(Duration::zero()),
          dur,
        )?;
      }
    }
  }
  Ok(totals.into_iter().collect())
}

fn bucket_by_day(
  entries: &[crate::model::TimeEntry],
) -> anyhow::Result<Vec<(String, Duration)>> {
  use std::collections::HashMap;

  let mut totals: HashMap<String, Duration> = HashMap::new();
  for e in entries {
    let key = DateTime::<Local>::from(e.start)
      .format("%Y-%m-%d")
      .to_string();
    add_into(
      totals.entry(key).or_insert(Duration::zero()),
      entry_duration(e),
    )?;
  }
  Ok(totals.into_iter().collect())
}
