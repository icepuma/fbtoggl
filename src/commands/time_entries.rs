use crate::{
  cli::{
    CreateTimeEntry, EditTimeEntry, Format, StartTimeEntry, StopTimeEntry,
    TimeEntryDetails, output_values_json,
  },
  client::TogglClient,
  commands::common::find_project_by_name,
  model::{Client, Project, Range, TimeEntry, Workspace},
  types::{ClientId, ProjectId, TimeEntryId, WorkspaceId},
};
use anyhow::{Context, anyhow};
use chrono::{DateTime, Duration, Local, NaiveDate, Utc};
use colored::Colorize;
use core::ops::Div;
use hhmmss::Hhmmss;
use itertools::Itertools;
use std::collections::HashMap;
use term_table::{
  Table, TableStyle, row::Row, table_cell::Alignment, table_cell::TableCell,
};

struct OutputEntry<'a> {
  id: TimeEntryId,
  date: NaiveDate,
  duration: Duration,
  workspace: &'a str,
  project: &'a str,
  client: &'a str,
  description: &'a str,
  billable: bool,
}

pub fn list(
  debug: bool,
  format: &Format,
  range: &Range,
  missing: bool,
  client: &TogglClient,
) -> anyhow::Result<()> {
  let mut time_entries = client
    .get_time_entries(debug, range)
    .context("Failed to fetch time entries from Toggl")?;

  if missing {
    let missing_datetimes = if time_entries.is_empty() {
      range.get_datetimes()?
    } else {
      let mut missing_datetimes = vec![];

      for date in range.get_datetimes()? {
        if !time_entries
          .iter()
          .map(|entry| DateTime::<Local>::from(entry.start).date_naive())
          .any(|x| x == date.date_naive())
        {
          missing_datetimes.push(date);
        }
      }

      missing_datetimes
    };

    if missing_datetimes.is_empty() {
      println!("No entries found!");
      return Ok(());
    }

    match format {
      Format::Json => output_values_json(&missing_datetimes),
      Format::Raw => output_missing_days_raw(&missing_datetimes),
      Format::Table => output_missing_days_table(&missing_datetimes),
    }
  } else if time_entries.is_empty() {
    println!("No entries found!");
    return Ok(());
  } else {
    let workspaces = client.get_workspaces(debug)?;
    let me = client.get_me(debug)?;

    let workspace_id = me.default_workspace_id;

    let projects = client.get_workspace_projects(debug, false, workspace_id)?;
    let clients = client
      .get_workspace_clients(debug, false, workspace_id)?
      .unwrap_or_else(|| {
        if debug {
          println!("No clients found for workspace {workspace_id}");
        }
        Vec::new()
      });

    let output_entries = collect_output_entries(
      &mut time_entries,
      &workspaces,
      &projects,
      &clients,
    );

    match format {
      Format::Json => output_values_json(&time_entries),
      Format::Raw => output_values_raw(&output_entries),
      Format::Table => output_values_table(&output_entries),
    }
  }

  Ok(())
}

fn collect_output_entries<'a>(
  values: &'a mut [TimeEntry],
  workspaces: &'a [Workspace],
  projects: &'a [Project],
  clients: &'a [Client],
) -> Vec<OutputEntry<'a>> {
  let workspace_lookup = workspaces
    .iter()
    .map(|workspace| (workspace.id, workspace))
    .collect::<HashMap<WorkspaceId, &Workspace>>();

  let project_lookup = projects
    .iter()
    .map(|project| (project.id, project))
    .collect::<HashMap<ProjectId, &Project>>();

  let client_lookup = clients
    .iter()
    .map(|client| (client.id, client))
    .collect::<HashMap<ClientId, &Client>>();

  let mut output_entries = vec![];

  values.sort_by(|e1, e2| e1.start.cmp(&e2.start));

  for entry in values {
    let maybe_workspace = workspace_lookup.get(&entry.wid);
    let maybe_project = &entry.pid.and_then(|pid| project_lookup.get(&pid));
    let maybe_client = maybe_project
      .and_then(|project| project.cid.and_then(|c| client_lookup.get(&c)));

    // Running (Started, but not stopped) time_entries have a negative duration
    let duration = if entry.duration.is_negative() {
      Duration::zero()
    } else {
      Duration::try_seconds(entry.duration).unwrap_or_default()
    };

    output_entries.push(OutputEntry {
      id: entry.id,
      date: entry.start.date_naive(),
      duration,
      workspace: maybe_workspace.map_or("-", |w| w.name.as_str()),
      project: maybe_project.map(|p| p.name.as_str()).unwrap_or("-"),
      client: maybe_client.map_or("-", |c| c.name.as_str()),
      description: entry.description.as_deref().unwrap_or(""),
      billable: entry.billable.unwrap_or_default(),
    });
  }

  output_entries
}

#[allow(
  clippy::arithmetic_side_effects,
  reason = "Date and duration arithmetic is necessary for time entry creation"
)]
pub fn create(
  debug: bool,
  format: &Format,
  time_entry: &CreateTimeEntry,
  client: &TogglClient,
) -> anyhow::Result<()> {
  let me = client.get_me(debug)?;
  let workspace_id = me.default_workspace_id;
  let projects = client.get_workspace_projects(debug, false, workspace_id)?;

  let project = find_project_by_name(&projects, &time_entry.project)?;

  let duration = calculate_duration(time_entry)?;

  if time_entry.lunch_break {
    let start = time_entry.start;
    let duration = duration.div(2);

    client.create_time_entry(
      debug,
      time_entry.description.as_deref(),
      workspace_id,
      time_entry.tags.as_deref(),
      duration,
      start,
      project.id,
      time_entry.non_billable,
    )?;

    let new_start = start + lunch_break() + duration;

    client.create_time_entry(
      debug,
      time_entry.description.as_deref(),
      workspace_id,
      time_entry.tags.as_deref(),
      duration,
      new_start,
      project.id,
      time_entry.non_billable,
    )?;
  } else {
    client.create_time_entry(
      debug,
      time_entry.description.as_deref(),
      workspace_id,
      time_entry.tags.as_deref(),
      duration,
      time_entry.start,
      project.id,
      time_entry.non_billable,
    )?;
  }

  list(debug, format, &Range::Today, false, client)?;

  Ok(())
}

const fn lunch_break() -> Duration {
  Duration::hours(1)
}

#[allow(
  clippy::arithmetic_side_effects,
  reason = "Duration calculations are necessary for time entry logic"
)]
pub(super) fn calculate_duration(
  time_entry: &CreateTimeEntry,
) -> anyhow::Result<Duration> {
  if let Some(end) = time_entry.end {
    let start = time_entry.start;

    if start >= end {
      return Err(anyhow!(
        "start='{start}' is greater or equal than end='{end}'"
      ));
    }

    let duration = end - start;

    if time_entry.lunch_break {
      calculate_duration_with_lunch_break(duration)
    } else {
      Ok(duration)
    }
  } else {
    time_entry
      .duration
      .ok_or_else(|| anyhow!("Please use either --duration or --end"))
  }
}

#[allow(
  clippy::arithmetic_side_effects,
  reason = "Duration subtraction is safe after checking duration > lunch"
)]
fn calculate_duration_with_lunch_break(
  duration: Duration,
) -> anyhow::Result<Duration> {
  let lunch = lunch_break();
  let duration_with_lunch_break = if duration > lunch {
    duration - lunch
  } else {
    return Err(anyhow!("Duration minus lunch break is <= 0"));
  };

  if duration_with_lunch_break <= Duration::zero() {
    Err(anyhow!("Duration minus lunch break is <= 0"))
  } else {
    Ok(duration_with_lunch_break)
  }
}

pub fn start(
  debug: bool,
  format: &Format,
  time_entry: &StartTimeEntry,
  client: &TogglClient,
) -> anyhow::Result<()> {
  let me = client.get_me(debug)?;
  let workspace_id = me.default_workspace_id;
  let projects = client.get_workspace_projects(debug, false, workspace_id)?;

  let project = find_project_by_name(&projects, &time_entry.project)?;

  let started_time_entry = client.start_time_entry(
    debug,
    chrono::Local::now(),
    workspace_id,
    time_entry.description.as_deref(),
    time_entry.tags.as_deref(),
    project.id,
    time_entry.non_billable,
  )?;

  match format {
    Format::Json => output_values_json(&[started_time_entry]),
    Format::Raw => output_time_entry_raw(&started_time_entry),
    Format::Table => output_time_entry_table(&started_time_entry),
  }

  Ok(())
}

pub fn stop(
  debug: bool,
  format: &Format,
  time_entry: &StopTimeEntry,
  client: &TogglClient,
) -> anyhow::Result<()> {
  let me = client.get_me(debug)?;
  let workspace_id = me.default_workspace_id;

  // If ID is provided, use it; otherwise this shouldn't be called
  if let Some(id) = time_entry.id {
    client.stop_time_entry(debug, workspace_id, id)?;
  } else {
    return Err(anyhow!("No time entry ID provided"));
  }

  list(debug, format, &Range::Today, false, client)?;

  Ok(())
}

pub fn delete(
  debug: bool,
  format: &Format,
  time_entry: TimeEntryDetails,
  client: &TogglClient,
) -> anyhow::Result<()> {
  client.delete_time_entry(debug, time_entry.id)?;

  list(debug, format, &Range::Today, false, client)?;

  Ok(())
}

pub fn details(
  debug: bool,
  format: &Format,
  time_entry: TimeEntryDetails,
  client: &TogglClient,
) -> anyhow::Result<()> {
  let entry = client.get_time_entry(debug, time_entry.id)?;

  let workspaces = client.get_workspaces(debug)?;
  let me = client.get_me(debug)?;
  let workspace_id = me.default_workspace_id;

  let projects = client.get_workspace_projects(debug, false, workspace_id)?;
  let clients = client
    .get_workspace_clients(debug, false, workspace_id)?
    .unwrap_or_default();

  let mut entries = vec![entry];
  let output_entries =
    collect_output_entries(&mut entries, &workspaces, &projects, &clients);

  match format {
    Format::Json => output_values_json(&entries),
    Format::Raw => output_values_raw(&output_entries),
    Format::Table => output_values_table(&output_entries),
  }

  Ok(())
}

pub fn stop_current(
  debug: bool,
  format: &Format,
  client: &TogglClient,
) -> anyhow::Result<()> {
  let me = client.get_me(debug)?;
  let workspace_id = me.default_workspace_id;

  // Get current running timer
  let current = client.get_current_time_entry(debug)?;

  if let Some(entry) = current {
    client.stop_time_entry(debug, workspace_id, entry.id)?;
    println!("Stopped timer: {}", entry.description.unwrap_or_default());
  } else {
    println!("No timer is currently running");
  }

  list(debug, format, &Range::Today, false, client)?;

  Ok(())
}

pub fn current(
  debug: bool,
  format: &Format,
  client: &TogglClient,
) -> anyhow::Result<()> {
  let current = client.get_current_time_entry(debug)?;

  if let Some(entry) = current {
    match format {
      Format::Json => output_values_json(&[entry]),
      Format::Raw => output_time_entry_raw(&entry),
      Format::Table => output_time_entry_table(&entry),
    }
  } else {
    println!("No timer is currently running");
  }

  Ok(())
}

pub fn continue_timer(
  debug: bool,
  format: &Format,
  id: Option<TimeEntryId>,
  client: &TogglClient,
) -> anyhow::Result<()> {
  let me = client.get_me(debug)?;
  let workspace_id = me.default_workspace_id;

  let entry_to_continue = if let Some(id) = id {
    // Continue specific entry
    client.get_time_entry(debug, id)?
  } else {
    // Continue last entry
    let entries = client.get_time_entries(debug, &Range::Today)?;
    entries
      .into_iter()
      .filter(|e| e.stop.is_some())
      .max_by_key(|e| e.stop)
      .ok_or_else(|| anyhow!("No completed time entries found today"))?
  };

  // Start new entry with same details
  let started = client.start_time_entry(
    debug,
    Local::now(),
    workspace_id,
    entry_to_continue.description.as_deref(),
    entry_to_continue.tags.as_deref(),
    entry_to_continue
      .pid
      .ok_or_else(|| anyhow!("Entry has no project"))?,
    !entry_to_continue.billable.unwrap_or(false),
  )?;

  match format {
    Format::Json => output_values_json(&[started]),
    Format::Raw => output_time_entry_raw(&started),
    Format::Table => output_time_entry_table(&started),
  }

  Ok(())
}

pub fn edit(
  debug: bool,
  format: &Format,
  edit_entry: &EditTimeEntry,
  client: &TogglClient,
) -> anyhow::Result<()> {
  // Get existing entry
  let mut entry = client.get_time_entry(debug, edit_entry.id)?;

  // Update fields if provided
  if let Some(ref project_name) = edit_entry.project {
    let me = client.get_me(debug)?;
    let workspace_id = me.default_workspace_id;
    let projects = client.get_workspace_projects(debug, false, workspace_id)?;
    let project = find_project_by_name(&projects, project_name)?;
    entry.pid = Some(project.id);
  }

  if let Some(ref desc) = edit_entry.description {
    entry.description = Some(desc.clone());
  }

  if let Some(ref tags) = edit_entry.tags {
    entry.tags = Some(tags.clone());
  }

  if let Some(start) = edit_entry.start {
    entry.start = start.with_timezone(&Utc);
  }

  if let Some(end) = edit_entry.end {
    entry.stop = Some(end.with_timezone(&Utc));
  }

  if edit_entry.toggle_billable {
    entry.billable = Some(!entry.billable.unwrap_or(false));
  }

  // Update the entry
  let updated = client.update_time_entry(debug, edit_entry.id, &entry)?;

  match format {
    Format::Json => output_values_json(&[updated]),
    Format::Raw => output_time_entry_raw(&updated),
    Format::Table => output_time_entry_table(&updated),
  }

  Ok(())
}

fn output_time_entry_raw(time_entry: &TimeEntry) {
  println!(
    "{}\t{}\t{}\t{}",
    &time_entry.id,
    &time_entry.start,
    time_entry.description.as_deref().unwrap_or(""),
    &time_entry
      .tags
      .as_ref()
      .map(|tags| tags.join(", "))
      .unwrap_or_default(),
  );
}

fn output_time_entry_table(time_entry: &TimeEntry) {
  let mut table = Table::new();
  table.style = TableStyle::thin();
  table.separate_rows = false;

  let header = Row::new(vec![
    TableCell::new("Id".bold().underline()),
    TableCell::new("Start".bold().underline()),
    TableCell::new("Description".bold().underline()),
    TableCell::new("Tags".bold().underline()),
  ]);

  table.add_row(header);

  table.add_row(Row::new(vec![
    TableCell::new(time_entry.id),
    TableCell::new(time_entry.start),
    TableCell::new(time_entry.description.as_deref().unwrap_or("")),
    TableCell::new(
      time_entry
        .tags
        .as_ref()
        .map(|tags| tags.join(", "))
        .unwrap_or_default(),
    ),
  ]));

  println!("{}", table.render());
}

fn output_missing_days_table(missing_datetimes: &[DateTime<Local>]) {
  let mut table = Table::new();
  table.style = TableStyle::thin();
  table.separate_rows = false;

  let header = Row::new(vec![TableCell::new("Date".bold().underline())]);

  table.add_row(header);

  for missing_datetime in missing_datetimes {
    table.add_row(Row::new(vec![TableCell::new(
      missing_datetime.date_naive(),
    )]));
  }

  println!("{}", table.render());
}

fn output_missing_days_raw(missing_datetimes: &[DateTime<Local>]) {
  for missing_datetime in missing_datetimes {
    println!("{}", missing_datetime.date_naive());
  }
}

fn output_values_raw(output_entries: &[OutputEntry<'_>]) {
  for entry in output_entries {
    let duration_text = if entry.duration.is_zero() {
      "running ".to_owned()
    } else {
      entry.duration.hhmmss()
    };

    println!(
      "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
      &entry.date,
      duration_text,
      &entry.id,
      &entry.workspace,
      &entry.project,
      &entry.client,
      &entry.description,
      if entry.billable {
        "BILLABLE"
      } else {
        "NON_BILLABLE"
      }
    );
  }
}

#[allow(
  clippy::too_many_lines,
  reason = "This function handles complex table formatting - splitting would reduce readability"
)]
fn output_values_table(output_entries: &[OutputEntry<'_>]) {
  let time_entry_buckets = output_entries
    .iter()
    .chunk_by(|e| &e.date)
    .into_iter()
    .map(|(date, group)| (date, group.collect()))
    .collect::<Vec<(&NaiveDate, Vec<&OutputEntry>)>>();

  if time_entry_buckets.is_empty() {
    println!("No entries found");
  } else {
    let mut table = Table::new();
    table.style = TableStyle::thin();
    table.separate_rows = false;

    let header = Row::new(vec![
      TableCell::new("Date".bold().underline()),
      TableCell::new("Time".bold().underline()),
      TableCell::new("Id".bold().underline()),
      TableCell::new("Workspace".bold().underline()),
      TableCell::new("Project".bold().underline()),
      TableCell::new("Customer".bold().underline()),
      TableCell::new("Description".bold().underline()),
      TableCell::new("Billable".bold().underline()),
    ]);

    table.add_row(header);

    table.add_row(Row::new(vec![
      TableCell::new(""),
      TableCell::new(""),
      TableCell::new(""),
      TableCell::new(""),
      TableCell::new(""),
      TableCell::new(""),
      TableCell::new(""),
      TableCell::new(""),
    ]));

    let mut total_time_sum: i64 = 0;

    for (date, entries) in time_entry_buckets {
      let time_sum: i64 =
        entries.iter().map(|e| e.duration.num_seconds()).sum();

      total_time_sum = total_time_sum.saturating_add(time_sum);

      let date_row = Row::new(vec![
        TableCell::new(date.to_string().bold()),
        TableCell::new(
          Duration::try_seconds(time_sum)
            .unwrap_or_default()
            .hhmmss()
            .bold(),
        ),
        TableCell::new(""),
        TableCell::new(""),
        TableCell::new(""),
        TableCell::new(""),
        TableCell::new(""),
        TableCell::new(""),
      ]);

      table.add_row(date_row);

      for entry in entries {
        let duration_text = if entry.duration.is_zero() {
          "running".italic()
        } else {
          entry.duration.hhmmss().italic()
        };

        let entry_row = Row::new(vec![
          TableCell::new(""),
          TableCell::new(duration_text),
          TableCell::new(entry.id),
          TableCell::new(entry.workspace),
          TableCell::new(entry.project),
          TableCell::new(entry.client),
          TableCell::new(entry.description),
          TableCell::builder(if entry.billable {
            "$".bold().green()
          } else {
            "$".bold().red()
          })
          .col_span(1)
          .alignment(Alignment::Center)
          .build(),
        ]);

        table.add_row(entry_row);
      }
    }

    table.add_row(Row::new(vec![
      TableCell::new(""),
      TableCell::new(""),
      TableCell::new(""),
      TableCell::new(""),
      TableCell::new(""),
      TableCell::new(""),
    ]));

    let total_sum_row = Row::new(vec![
      TableCell::new("Total".bold()),
      TableCell::new(
        Duration::try_seconds(total_time_sum)
          .unwrap_or_default()
          .hhmmss()
          .bold()
          .underline(),
      ),
      TableCell::new(""),
      TableCell::new(""),
      TableCell::new(""),
      TableCell::new(""),
    ]);

    table.add_row(total_sum_row);

    println!("{}", table.render());
  }
}
