use crate::{
  cli::{
    output_values_json, CreateTimeEntry, Format, StartTimeEntry, StopTimeEntry,
  },
  client::TogglClient,
  model::{Client, Project, Range, TimeEntry, Workspace},
};
use anyhow::anyhow;
use chrono::{Duration, NaiveDate};
use colored::Colorize;
use hhmmss::Hhmmss;
use itertools::Itertools;
use std::{collections::HashMap, ops::Div};
use term_table::{row::Row, table_cell::TableCell, Table, TableStyle};

struct OutputEntry {
  date: NaiveDate,
  duration: Duration,
  workspace: String,
  project: String,
  client: String,
  description: String,
}

pub fn list(
  format: &Format,
  range: &Range,
  client: &TogglClient,
) -> anyhow::Result<()> {
  let mut time_entries = client.get_time_entries(range)?;

  if time_entries.is_empty() {
    println!("No entries found!");
    return Ok(());
  }

  let workspaces = client.get_workspaces()?;
  let me = client.get_me()?;

  let workspace_id = me.data.default_wid;

  let projects = client.get_workspace_projects(workspace_id)?;
  let clients = client.get_workspace_clients(workspace_id)?;

  let output_entries =
    collect_output_entries(&mut time_entries, &workspaces, &projects, &clients);

  match format {
    Format::Json => output_values_json(&time_entries),
    Format::Raw => output_values_raw(&output_entries),
    Format::Table => output_values_table(&output_entries),
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
    let maybe_project = project_lookup.get(&entry.pid);
    let maybe_client = maybe_project
      .and_then(|project| project.cid.and_then(|c| client_lookup.get(&c)));

    // Running (Started, but not stopped) time_entries have a negative duration
    let duration = if entry.duration.is_negative() {
      Duration::zero()
    } else {
      Duration::seconds(entry.duration)
    };

    output_entries.push(OutputEntry {
      date: entry.start.date().naive_local(),
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
    })
  }

  output_entries
}

pub fn create(
  format: &Format,
  time_entry: &CreateTimeEntry,
  client: &TogglClient,
) -> anyhow::Result<()> {
  let me = client.get_me()?;
  let workspace_id = me.data.default_wid;
  let projects = client.get_workspace_projects(workspace_id)?;

  let project = projects
    .iter()
    .find(|project| project.name == time_entry.project)
    .ok_or(anyhow!(format!(
      "Cannot find project='{}'",
      time_entry.project
    )))?;

  if time_entry.lunch_break {
    let start = time_entry.start.as_date_time();
    let duration = time_entry.duration.div(2);

    client.create_time_entry(
      &time_entry.description,
      workspace_id,
      &time_entry.tags,
      duration,
      start,
      project.id,
    )?;

    let new_start = start + Duration::hours(1) + duration;

    client.create_time_entry(
      &time_entry.description,
      workspace_id,
      &time_entry.tags,
      duration,
      new_start,
      project.id,
    )?;
  } else {
    client.create_time_entry(
      &time_entry.description,
      workspace_id,
      &time_entry.tags,
      time_entry.duration,
      time_entry.start.as_date_time(),
      project.id,
    )?;
  }

  list(format, &Range::Today, client)?;

  Ok(())
}

pub fn start(
  format: &Format,
  time_entry: &StartTimeEntry,
  client: &TogglClient,
) -> anyhow::Result<()> {
  let me = client.get_me()?;
  let workspace_id = me.data.default_wid;
  let projects = client.get_workspace_projects(workspace_id)?;

  let project = projects
    .iter()
    .find(|project| project.name == time_entry.project)
    .ok_or(anyhow!(format!(
      "Cannot find project='{}'",
      time_entry.project
    )))?;

  let started_time_entry = client.start_time_entry(
    &time_entry.description,
    &time_entry.tags,
    project.id,
  )?;

  match format {
    Format::Json => output_values_json(&[started_time_entry.data]),
    Format::Raw => output_time_entry_raw(&started_time_entry.data),
    Format::Table => output_time_entry_table(&started_time_entry.data),
  }

  Ok(())
}

pub fn stop(
  format: &Format,
  time_entry: &StopTimeEntry,
  client: &TogglClient,
) -> anyhow::Result<()> {
  let me = client.get_me()?;
  let workspace_id = me.data.default_wid;
  let projects = client.get_workspace_projects(workspace_id)?;

  let project = projects
    .iter()
    .find(|project| project.name == time_entry.project)
    .ok_or(anyhow!(format!(
      "Cannot find project='{}'",
      time_entry.project
    )))?;

  client.stop_time_entry(
    time_entry.id,
    &time_entry.description,
    &time_entry.tags,
    project.id,
  )?;

  list(format, &Range::Today, client)?;

  Ok(())
}

fn output_time_entry_raw(time_entry: &TimeEntry) {
  println!(
    "{}\t{}\t{}\t{}",
    &time_entry.id,
    &time_entry.start,
    &time_entry.description.to_owned().unwrap_or_default(),
    &time_entry.tags.join(", "),
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
    TableCell::new(&time_entry.id),
    TableCell::new(&time_entry.start),
    TableCell::new(&time_entry.description.to_owned().unwrap_or_default()),
    TableCell::new(&time_entry.tags.join(", ")),
  ]));

  println!("{}", table.render());
}

fn output_values_raw(output_entries: &[OutputEntry]) {
  for entry in output_entries {
    let duration_text = if entry.duration.is_zero() {
      "running ".to_string()
    } else {
      entry.duration.hhmmss()
    };

    println!(
      "{}\t{}\t{}\t{}\t{}\t{}",
      &entry.date,
      duration_text,
      &entry.workspace,
      &entry.project,
      &entry.client,
      &entry.description
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
      TableCell::new("Workspace".bold().underline()),
      TableCell::new("Project".bold().underline()),
      TableCell::new("Customer".bold().underline()),
      TableCell::new("Description".bold().underline()),
    ]);

    table.add_row(header);

    table.add_row(Row::new(vec![
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
          TableCell::new(&entry.workspace),
          TableCell::new(&entry.project),
          TableCell::new(&entry.client),
          TableCell::new(&entry.description),
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
