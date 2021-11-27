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

    output_entries.push(OutputEntry {
      date: entry.start.date().naive_local(),
      duration: Duration::seconds(entry.duration),
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
    println!(
      "{}\t{}\t{}\t{}\t{}\t{}",
      &entry.date,
      &entry.duration.hhmmss(),
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
        let entry_row = Row::new(vec![
          TableCell::new(""),
          TableCell::new(&entry.duration.hhmmss().italic()),
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

#[cfg(test)]
mod tests {
  use crate::{
    cli::CreateTimeEntry,
    client::{TogglClient, CREATED_WITH},
    commands::time_entries::create,
    model::Start,
  };
  use chrono::{DateTime, Duration, Local};
  use mockito::{mock, Matcher};
  use serde_json::{json, Value};
  use std::str::FromStr;

  #[ctor::ctor]
  fn setup() {
    std::env::set_var("RUST_LOG", "mockito=debug");
    std::env::set_var("TZ", "Europe/Berlin");

    let _ = env_logger::try_init();
  }

  #[ctor::dtor]
  fn teardown() {
    std::env::remove_var("RUST_LOG");
    std::env::remove_var("TZ");
  }

  #[test]
  fn test_create_workday_with_pause_2_hours() -> anyhow::Result<()> {
    let me_mock = mock("GET", "/me")
      .with_header(
        "Authorization",
        "Basic Y2I3YmY3ZWZhNmQ2NTIwNDZhYmQyZjdkODRlZTE4YzE6YXBpX3Rva2Vu",
      )
      .with_status(200)
      .with_body(me().to_string())
      .expect(1)
      .create();

    let projects_mock = mock("GET", "/workspaces/1234567/projects")
      .with_header(
        "Authorization",
        "Basic Y2I3YmY3ZWZhNmQ2NTIwNDZhYmQyZjdkODRlZTE4YzE6YXBpX3Rva2Vu",
      )
      .with_status(200)
      .with_body(projects().to_string())
      .create();

    let request_body = json!(
      {
        "time_entry": {
          "description": "fkbr",
          "wid": 1234567,
          "duration": 7200,
          "start": "2021-11-21T23:58:09+01:00",
          "tags": null,
          "pid": 123456789,
          "created_with": CREATED_WITH,
        }
      }
    );

    let response_body = json!(
      {
        "data": {
          "id": 1234567890,
          "wid": 1234567,
          "pid": 123456789,
          "billable": false,
          "start": "2021-11-21T23:58:09+01:00",
          "duration": 200,
          "description": "fkbr",
          "duronly": false,
          "at": "2021-11-21T23:58:09+01:00",
          "uid": 123456789
        }
      }
    );

    let time_entry_create_mock = mock("POST", "/time_entries")
      .with_header(
        "Authorization",
        "Basic Y2I3YmY3ZWZhNmQ2NTIwNDZhYmQyZjdkODRlZTE4YzE6YXBpX3Rva2Vu",
      )
      .with_status(200)
      .match_body(Matcher::Json(request_body))
      .with_body(response_body.to_string())
      .expect(1)
      .create();

    let list_entries_mock =
      mock("GET", Matcher::Regex(r"^/time_entries.*$".to_string()))
        .with_header(
          "Authorization",
          "Basic Y2I3YmY3ZWZhNmQ2NTIwNDZhYmQyZjdkODRlZTE4YzE6YXBpX3Rva2Vu",
        )
        .with_status(200)
        .expect(1)
        .with_body("[]")
        .create();

    {
      let workday_with_pause = CreateTimeEntry {
        description: "fkbr".to_string(),
        start: Start::Date(DateTime::<Local>::from_str(
          "2021-11-21T22:58:09Z",
        )?),
        duration: Duration::hours(2),
        lunch_break: false,
        project: "betamale gmbh".to_string(),
        tags: None,
      };

      let client =
        TogglClient::new("cb7bf7efa6d652046abd2f7d84ee18c1".to_string())?;

      create(&crate::cli::Format::Json, &workday_with_pause, &client)?;
    }

    me_mock.assert();
    projects_mock.assert();
    time_entry_create_mock.assert();
    list_entries_mock.assert();

    Ok(())
  }

  #[test]
  fn test_create_workday_with_pause_7_hours() -> anyhow::Result<()> {
    let me_mock = mock("GET", "/me")
      .with_header(
        "Authorization",
        "Basic Y2I3YmY3ZWZhNmQ2NTIwNDZhYmQyZjdkODRlZTE4YzE6YXBpX3Rva2Vu",
      )
      .with_status(200)
      .with_body(me().to_string())
      .expect(1)
      .create();

    let projects_mock = mock("GET", "/workspaces/1234567/projects")
      .with_header(
        "Authorization",
        "Basic Y2I3YmY3ZWZhNmQ2NTIwNDZhYmQyZjdkODRlZTE4YzE6YXBpX3Rva2Vu",
      )
      .with_status(200)
      .with_body(projects().to_string())
      .create();

    let first_request_body = json!(
      {
        "time_entry": {
          "description": "fkbr",
          "wid": 1234567,
          "duration": 12600,
          "start": "2021-11-21T22:58:09+01:00",
          "tags": null,
          "pid": 123456789,
          "created_with": CREATED_WITH,
        }
      }
    );

    let first_response_body = json!(
      {
        "data": {
          "id": 1234567890,
          "wid": 1234567,
          "pid": 123456789,
          "billable": false,
          "start": "2021-11-21T22:58:09+01:00",
          "duration": 12600,
          "description": "fkbr",
          "duronly": false,
          "at": "2021-11-21T22:58:09+01:00",
          "uid": 123456789
        }
      }
    );

    let second_request_body = json!(
      {
        "time_entry": {
          "description": "fkbr",
          "wid": 1234567,
          "duration": 12600,
          "start": "2021-11-22T03:28:09+01:00",
          "tags": null,
          "pid": 123456789,
          "created_with": CREATED_WITH,
        }
      }
    );

    let second_response_body = json!(
      {
        "data": {
          "id": 1234567890,
          "wid": 1234567,
          "pid": 123456789,
          "billable": false,
          "start": "2021-11-22T03:28:09+01:00",
          "duration": 12600,
          "description": "fkbr",
          "duronly": false,
          "at": "2021-11-22T03:28:09+01:00",
          "uid": 123456789
        }
      }
    );

    let first_time_entry_create_mock = mock("POST", "/time_entries")
      .with_header(
        "Authorization",
        "Basic Y2I3YmY3ZWZhNmQ2NTIwNDZhYmQyZjdkODRlZTE4YzE6YXBpX3Rva2Vu",
      )
      .with_status(200)
      .with_body(first_response_body.to_string())
      .match_body(Matcher::Json(first_request_body))
      .expect(1)
      .create();

    let second_time_entry_create_mock = mock("POST", "/time_entries")
      .with_header(
        "Authorization",
        "Basic Y2I3YmY3ZWZhNmQ2NTIwNDZhYmQyZjdkODRlZTE4YzE6YXBpX3Rva2Vu",
      )
      .with_status(200)
      .with_body(second_response_body.to_string())
      .match_body(Matcher::Json(second_request_body))
      .expect(1)
      .create();

    let list_entries_mock =
      mock("GET", Matcher::Regex(r"^/time_entries.*$".to_string()))
        .with_header(
          "Authorization",
          "Basic Y2I3YmY3ZWZhNmQ2NTIwNDZhYmQyZjdkODRlZTE4YzE6YXBpX3Rva2Vu",
        )
        .with_status(200)
        .expect(1)
        .with_body("[]")
        .create();

    {
      let workday_with_pause = CreateTimeEntry {
        description: "fkbr".to_string(),
        start: Start::Date(DateTime::<Local>::from_str(
          "2021-11-21T22:58:09+01:00",
        )?),
        duration: Duration::hours(7),
        lunch_break: true,
        project: "betamale gmbh".to_string(),
        tags: None,
      };

      let client =
        TogglClient::new("cb7bf7efa6d652046abd2f7d84ee18c1".to_string())?;

      create(&crate::cli::Format::Json, &workday_with_pause, &client)?;
    }

    me_mock.assert();
    projects_mock.assert();
    first_time_entry_create_mock.assert();
    second_time_entry_create_mock.assert();
    list_entries_mock.assert();

    Ok(())
  }

  fn me() -> Value {
    json!(
      {
        "since": 1234567890,
        "data": {
          "id": 1234567,
          "api_token": "cb7bf7efa6d652046abd2f7d84ee18c1",
          "default_wid": 1234567,
          "email": "ralph.bower@fkbr.org",
          "fullname": "Ralph Bower",
          "jquery_timeofday_format": "H:i",
          "jquery_date_format": "Y-m-d",
          "timeofday_format": "H:mm",
          "date_format": "YYYY-MM-DD",
          "store_start_and_stop_time": true,
          "beginning_of_week": 1,
          "language": "en_US",
          "image_url": "https://assets.track.toggl.com/images/profile.png",
          "sidebar_piechart": true,
          "at": "2021-11-16T08:45:25+00:00",
          "created_at": "2021-11-16T08:41:05+00:00",
          "retention": 9,
          "record_timeline": false,
          "render_timeline": false,
          "timeline_enabled": false,
          "timeline_experiment": false,
          "should_upgrade": false,
          "timezone": "Europe/Berlin",
          "openid_enabled": false,
          "send_product_emails": true,
          "send_weekly_report": true,
          "send_timer_notifications": true,
          "invitation": {},
          "duration_format": "improved"
        }
      }
    )
  }

  fn projects() -> Value {
    json!(
      [
        {
          "id": 123456789,
          "wid": 1234567,
          "cid": 87654321,
          "name": "betamale gmbh",
          "billable": true,
          "is_private": true,
          "active": true,
          "template": false,
          "at": "2021-11-16T09:30:22+00:00",
          "created_at": "2021-11-16T09:30:22+00:00",
          "color": "5",
          "auto_estimates": false,
          "actual_hours": 4,
          "hex_color": "#2da608"
        },
        {
          "id": 987654321,
          "wid": 1234567,
          "cid": 12345678,
          "name": "fkbr.org",
          "billable": true,
          "is_private": false,
          "active": true,
          "template": false,
          "at": "2021-11-16T08:51:21+00:00",
          "created_at": "2021-11-16T08:42:34+00:00",
          "color": "14",
          "auto_estimates": false,
          "actual_hours": 23,
          "rate": 100,
          "currency": "EUR",
          "hex_color": "#525266"
        }
      ]
    )
  }
}
