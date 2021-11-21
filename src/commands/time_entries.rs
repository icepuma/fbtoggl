use std::ops::Div;

use super::init_client;
use crate::cli::{
  output_value, output_values, CreateTimeEntry, CreateWorkdayWithPause, Format,
};
use anyhow::anyhow;
use chrono::Duration;

pub fn list(format: &Format) -> anyhow::Result<()> {
  let client = init_client()?;
  let time_entries = client.get_time_entries()?;

  output_values(format, time_entries);

  Ok(())
}

pub fn create(
  format: &Format,
  time_entry: &CreateTimeEntry,
) -> anyhow::Result<()> {
  let client = init_client()?;
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

  let data = client.create_time_entry(
    &time_entry.description,
    &time_entry.tags,
    time_entry.duration,
    time_entry.start.as_date_time(),
    project.id,
  )?;

  output_value(format, data.data);

  Ok(())
}

pub fn create_workday_with_pause(
  time_entry: &CreateWorkdayWithPause,
) -> anyhow::Result<()> {
  let client = init_client()?;
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

  // https://karrierebibel.de/pausenregelung/
  let pause = if time_entry.hours >= 6.0 && time_entry.hours <= 9.0 {
    Duration::minutes(30)
  } else if time_entry.hours >= 9.0 {
    Duration::minutes(45)
  } else {
    Duration::minutes(0)
  };

  if pause.is_zero() {
    let duration = (time_entry.hours * 3600.0).ceil();

    client.create_time_entry(
      &time_entry.description,
      &None,
      Duration::seconds(duration as i64).num_seconds() as u64,
      time_entry.start.as_date_time(),
      project.id,
    )?;
  } else {
    let duration = (time_entry.hours.div(2.0) * 3600.0).ceil();

    client.create_time_entry(
      &time_entry.description,
      &None,
      Duration::seconds(duration as i64).num_seconds() as u64,
      time_entry.start.as_date_time(),
      project.id,
    )?;

    let new_start = time_entry.start.as_date_time()
      + Duration::seconds(duration as i64)
      + pause;

    client.create_time_entry(
      &time_entry.description,
      &None,
      Duration::seconds(duration as i64).num_seconds() as u64,
      new_start,
      project.id,
    )?;
  }

  Ok(())
}
