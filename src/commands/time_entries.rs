use crate::{
  cli::{
    output_values_json, CreateTimeEntry, DeleteTimeEntry, Format,
    StartTimeEntry, StopTimeEntry,
  },
  client::TogglClient,
  model::{Client, Project, Range, TimeEntry, Workspace},
};
use anyhow::anyhow;
use chrono::{DateTime, Duration, Local, NaiveDate};
use colored::Colorize;
use hhmmss::Hhmmss;
use itertools::Itertools;
use std::{collections::HashMap, ops::Div};
use term_table::{
  row::Row, table_cell::Alignment, table_cell::TableCell, Table, TableStyle,
};

struct OutputEntry {
  id: u64,
  date: NaiveDate,
  duration: Duration,
  workspace: String,
  project: String,
  client: String,
  description: String,
  billable: bool,
}

pub fn list(
  debug: bool,
  format: &Format,
  range: &Range,
  missing: bool,
  client: &TogglClient,
) -> anyhow::Result<()> {
  let mut time_entries = client.get_time_entries(debug, range)?;

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
  } else {
    if time_entries.is_empty() {
      println!("No entries found!");
      return Ok(());
    }

    let workspaces = client.get_workspaces(debug)?;
    let me = client.get_me(debug)?;

    let workspace_id = me.default_workspace_id;

    let projects = client.get_workspace_projects(debug, workspace_id)?;
    let clients = client
      .get_workspace_clients(debug, workspace_id)?
      .unwrap_or_default();

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

fn collect_output_entries(
  values: &mut [TimeEntry],
  workspaces: &[Workspace],
  projects: &[Project],
  clients: &[Client],
) -> Vec<OutputEntry> {
  let workspace_lookup = workspaces
    .iter()
    .map(|workspace| (workspace.id, workspace))
    .collect::<HashMap<u64, &Workspace>>();

  let project_lookup = projects
    .iter()
    .map(|project| (project.id, project))
    .collect::<HashMap<u64, &Project>>();

  let client_lookup = clients
    .iter()
    .map(|client| (client.id, client))
    .collect::<HashMap<u64, &Client>>();

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
      Duration::seconds(entry.duration)
    };

    output_entries.push(OutputEntry {
      id: entry.id,
      date: entry.start.date_naive(),
      duration,
      workspace: maybe_workspace
        .map(|w| w.name.to_owned())
        .unwrap_or_else(|| "-".to_string()),
      project: maybe_project
        .map(|p| p.name.to_owned())
        .unwrap_or_else(|| "-".to_string()),
      client: maybe_client
        .map(|c| c.name.to_owned())
        .unwrap_or_else(|| "-".to_string()),
      description: entry.description.to_owned().unwrap_or_default(),
      billable: entry.billable.unwrap_or_default(),
    })
  }

  output_entries
}

pub fn create(
  debug: bool,
  format: &Format,
  time_entry: &CreateTimeEntry,
  client: &TogglClient,
) -> anyhow::Result<()> {
  let me = client.get_me(debug)?;
  let workspace_id = me.default_workspace_id;
  let projects = client.get_workspace_projects(debug, workspace_id)?;

  let project = projects
    .iter()
    .find(|project| project.name == time_entry.project)
    .ok_or_else(|| {
      anyhow!(format!("Cannot find project='{}'", time_entry.project))
    })?;

  let duration = calculate_duration(time_entry)?;

  if time_entry.lunch_break {
    let start = time_entry.start;
    let duration = duration.div(2);

    client.create_time_entry(
      debug,
      &time_entry.description,
      workspace_id,
      &time_entry.tags,
      duration,
      start,
      project.id,
      time_entry.non_billable,
    )?;

    let new_start = start + launch_break() + duration;

    client.create_time_entry(
      debug,
      &time_entry.description,
      workspace_id,
      &time_entry.tags,
      duration,
      new_start,
      project.id,
      time_entry.non_billable,
    )?;
  } else {
    client.create_time_entry(
      debug,
      &time_entry.description,
      workspace_id,
      &time_entry.tags,
      duration,
      time_entry.start,
      project.id,
      time_entry.non_billable,
    )?;
  }

  list(debug, format, &Range::Today, false, client)?;

  Ok(())
}

fn launch_break() -> Duration {
  Duration::hours(1)
}

pub(super) fn calculate_duration(
  time_entry: &CreateTimeEntry,
) -> anyhow::Result<Duration> {
  if let Some(end) = time_entry.end {
    let start = time_entry.start;

    if start >= end {
      return Err(anyhow!(
        "start='{}' is greater or equal than end='{}'",
        start,
        end
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

fn calculate_duration_with_lunch_break(
  duration: Duration,
) -> anyhow::Result<Duration> {
  let duration_with_lunch_break = duration - launch_break();

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
  let projects = client.get_workspace_projects(debug, workspace_id)?;

  let project = projects
    .iter()
    .find(|project| project.name == time_entry.project)
    .ok_or_else(|| {
      anyhow!(format!("Cannot find project='{}'", time_entry.project))
    })?;

  let started_time_entry = client.start_time_entry(
    debug,
    chrono::Local::now(),
    workspace_id,
    &time_entry.description,
    &time_entry.tags,
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

  client.stop_time_entry(debug, workspace_id, time_entry.id)?;

  list(debug, format, &Range::Today, false, client)?;

  Ok(())
}

pub fn delete(
  debug: bool,
  format: &Format,
  time_entry: &DeleteTimeEntry,
  client: &TogglClient,
) -> anyhow::Result<()> {
  client.delete_time_entry(debug, time_entry.id)?;

  list(debug, format, &Range::Today, false, client)?;

  Ok(())
}

fn output_time_entry_raw(time_entry: &TimeEntry) {
  println!(
    "{}\t{}\t{}\t{}",
    &time_entry.id,
    &time_entry.start,
    &time_entry.description.to_owned().unwrap_or_default(),
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
    TableCell::new(&time_entry.description.to_owned().unwrap_or_default()),
    TableCell::new(
      &time_entry
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

fn output_values_raw(output_entries: &[OutputEntry]) {
  for entry in output_entries {
    let duration_text = if entry.duration.is_zero() {
      "running ".to_string()
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

fn output_values_table(output_entries: &[OutputEntry]) {
  let time_entry_buckets = output_entries
    .iter()
    .group_by(|e| &e.date)
    .into_iter()
    .map(|(date, group)| (date, group.collect()))
    .collect::<Vec<(&NaiveDate, Vec<&OutputEntry>)>>();

  if !time_entry_buckets.is_empty() {
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

    let mut total_time_sum = 0;

    for (date, entries) in time_entry_buckets {
      let time_sum: i64 =
        entries.iter().map(|e| e.duration.num_seconds()).sum();

      total_time_sum += time_sum;

      let date_row = Row::new(vec![
        TableCell::new(date.to_string().bold()),
        TableCell::new(Duration::seconds(time_sum).hhmmss().bold()),
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
          TableCell::new(&entry.workspace),
          TableCell::new(&entry.project),
          TableCell::new(&entry.client),
          TableCell::new(&entry.description),
          TableCell::new_with_alignment(
            if entry.billable {
              "$".bold().green()
            } else {
              "$".bold().red()
            },
            1,
            Alignment::Center,
          ),
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
        Duration::seconds(total_time_sum)
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
  } else {
    println!("No entries found");
  }
}
