#![allow(
  clippy::unreadable_literal,
  reason = "JSON test data contains literal numbers that are more readable without separators"
)]
#![allow(clippy::unwrap_used, reason = "Test code can panic on failure")]
#![allow(
  clippy::significant_drop_tightening,
  reason = "Test server must remain alive for the duration of tests"
)]

use crate::{
  client::TogglClient,
  common::CREATED_WITH,
  model::Range,
  types::{ClientId, ProjectId, TimeEntryId, WorkspaceId},
};
use chrono::{DateTime, Duration, Local, NaiveDate};
use core::str::FromStr;
use mockito::Matcher;
use pretty_assertions::assert_eq;
use serde_json::json;

#[ctor::ctor]
fn setup() {
  unsafe {
    std::env::set_var("RUST_LOG", "mockito=debug");
    std::env::set_var("TZ", "Europe/Berlin");
  }

  let _ = env_logger::try_init();
}

#[ctor::dtor]
fn teardown() {
  unsafe {
    std::env::remove_var("RUST_LOG");
    std::env::remove_var("TZ");
  }
}

#[test]
fn get_me() -> anyhow::Result<()> {
  let mut server = mockito::Server::new();

  let body = json!(
    {
      "default_workspace_id": 1234567,
    }
  );

  let mock = server
    .mock("GET", "/me")
    .with_header(
      "Authorization",
      "Basic Y2I3YmY3ZWZhNmQ2NTIwNDZhYmQyZjdkODRlZTE4YzE6YXBpX3Rva2Vu",
    )
    .with_status(200)
    .with_body(body.to_string())
    .expect(1)
    .create();

  {
    let client = TogglClient::new_with_base_url(
      "cb7bf7efa6d652046abd2f7d84ee18c1".to_owned(),
      server.url().parse()?,
    )?;

    let me = client.get_me(false)?;

    assert_eq!(me.default_workspace_id, WorkspaceId::new(1234567));
  }

  mock.assert();

  Ok(())
}

#[test]
fn get_workspaces() -> anyhow::Result<()> {
  let body = json!(
    [
      {
        "id": 1234567,
        "name": "Ralph Bower Workspace",
        "profile": 102,
        "premium": true,
        "admin": true,
        "default_hourly_rate": 0,
        "default_currency": "EUR",
        "only_admins_may_create_projects": false,
        "only_admins_see_billable_rates": false,
        "only_admins_see_team_dashboard": false,
        "projects_billable_by_default": true,
        "rounding": 1,
        "rounding_minutes": 0,
        "api_token": "febb91e4d84e2aca80532c4bc0adce53",
        "at": "2021-11-16T08:52:59+00:00",
        "logo_url": "https://assets.toggl.com/images/workspace.jpg",
        "ical_url": "/ical/workspace_user/7d70663568cac5af684503681e3a4d41",
        "ical_enabled": true
      }
    ]
  );

  let mut server = mockito::Server::new();

  let mock = server
    .mock("GET", "/workspaces")
    .with_header(
      "Authorization",
      "Basic Y2I3YmY3ZWZhNmQ2NTIwNDZhYmQyZjdkODRlZTE4YzE6YXBpX3Rva2Vu",
    )
    .with_status(200)
    .with_body(body.to_string())
    .expect(1)
    .create();

  {
    let client = TogglClient::new_with_base_url(
      "cb7bf7efa6d652046abd2f7d84ee18c1".to_owned(),
      server.url().parse()?,
    )?;

    let workspaces = client.get_workspaces(false)?;
    let first_workspace = workspaces.first().unwrap();

    assert_eq!(first_workspace.id, WorkspaceId::new(1234567));
    assert_eq!(first_workspace.name, "Ralph Bower Workspace");
  }

  mock.assert();

  Ok(())
}

#[test]
fn get_workspace_clients() -> anyhow::Result<()> {
  let body = json!(
    [
      {
        "id": 1234,
        "wid": 12345678,
        "name": "fkbr.org",
        "at": "2021-11-16T09:30:21+00:00",
        "archived": false
      },
      {
        "id": 2345,
        "wid": 12345678,
        "name": "beta male gmbh",
        "at": "2021-11-16T08:42:34+00:00",
        "archived": false
      }
    ]
  );

  let mut server = mockito::Server::new();

  let mock = server
    .mock("GET", "/workspaces/12345678/clients?status=active")
    .with_header(
      "Authorization",
      "Basic Y2I3YmY3ZWZhNmQ2NTIwNDZhYmQyZjdkODRlZTE4YzE6YXBpX3Rva2Vu",
    )
    .with_status(200)
    .with_body(body.to_string())
    .expect(1)
    .create();

  {
    let client = TogglClient::new_with_base_url(
      "cb7bf7efa6d652046abd2f7d84ee18c1".to_owned(),
      server.url().parse()?,
    )?;

    let clients = client
      .get_workspace_clients(false, false, WorkspaceId::new(12345678))?
      .unwrap_or_default();
    let first_client = clients.first().unwrap();
    let second_client = clients.get(1).unwrap();

    assert_eq!(first_client.id, ClientId::new(1234));
    assert_eq!(first_client.name, "fkbr.org");

    assert_eq!(second_client.id, ClientId::new(2345));
    assert_eq!(second_client.name, "beta male gmbh");
  }

  mock.assert();

  Ok(())
}

#[test]
fn get_workspace_projects() -> anyhow::Result<()> {
  let body = json!(
    [
      {
        "id": 123456789,
        "wid": 1234567,
        "cid": 87654321,
        "name": "beta male gmbh",
        "billable": true,
        "is_private": true,
        "active": true,
        "template": false,
        "at": "2021-11-16T09:30:22+00:00",
        "created_at": "2021-11-16T09:30:22+00:00",
        "color": "5",
        "auto_estimates": false,
        "actual_hours": 4,
        "hex_color": "#2da608",
        "status": "active"
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
        "hex_color": "#525266",
        "status": "active"
      }
    ]
  );

  let mut server = mockito::Server::new();

  let mock = server
    .mock("GET", "/workspaces/12345678/projects?active=true")
    .with_header(
      "Authorization",
      "Basic Y2I3YmY3ZWZhNmQ2NTIwNDZhYmQyZjdkODRlZTE4YzE6YXBpX3Rva2Vu",
    )
    .with_status(200)
    .with_body(body.to_string())
    .expect(1)
    .create();

  {
    let client = TogglClient::new_with_base_url(
      "cb7bf7efa6d652046abd2f7d84ee18c1".to_owned(),
      server.url().parse()?,
    )?;

    let projects = client.get_workspace_projects(
      false,
      false,
      WorkspaceId::new(12345678),
    )?;
    let first_project = projects.first().unwrap();
    let second_project = projects.get(1).unwrap();

    assert_eq!(first_project.id, ProjectId::new(123456789));
    assert_eq!(first_project.wid, WorkspaceId::new(1234567));
    assert_eq!(first_project.name, "beta male gmbh");

    assert_eq!(second_project.id, ProjectId::new(987654321));
    assert_eq!(second_project.wid, WorkspaceId::new(1234567));
    assert_eq!(second_project.name, "fkbr.org");
  }

  mock.assert();

  Ok(())
}

#[test]
fn get_time_entries() -> anyhow::Result<()> {
  let body = json!(
    [
      {
        "id": 123456789,
        "guid": "6fbbba91487e5d911a6e51a7188e572e",
        "wid": 1234567,
        "pid": 123456789,
        "billable": true,
        "start": "2021-11-21T22:57:33+00:00",
        "stop": "2021-11-21T22:57:37+00:00",
        "duration": 4,
        "description": "Wurst",
        "duronly": false,
        "at": "2021-11-21T22:57:37+00:00",
        "uid": 1234567
      },
      {
        "id": 987654321,
        "guid": "159610d3532b1644e4b520f2f4f80943",
        "wid": 1234567,
        "pid": 123456789,
        "billable": true,
        "start": "2021-11-21T22:58:09+00:00",
        "stop": "2021-11-21T22:58:12+00:00",
        "duration": 3,
        "description": "Kaese",
        "duronly": false,
        "at": "2021-11-21T22:58:12+00:00",
        "uid": 1234567
      }
    ]
  );

  let mut server = mockito::Server::new();

  let mock = server
    .mock(
      "GET",
      "/me/time_entries?start_date=2021-11-21&end_date=2021-11-23",
    )
    .with_header(
      "Authorization",
      "Basic Y2I3YmY3ZWZhNmQ2NTIwNDZhYmQyZjdkODRlZTE4YzE6YXBpX3Rva2Vu",
    )
    .with_status(200)
    .with_body(body.to_string())
    .expect(1)
    .create();

  {
    let client = TogglClient::new_with_base_url(
      "cb7bf7efa6d652046abd2f7d84ee18c1".to_owned(),
      server.url().parse()?,
    )?;

    let time_entries = client.get_time_entries(
      false,
      &Range::Date(NaiveDate::from_ymd_opt(2021, 11, 21).unwrap()),
    )?;
    let first_time_entry = time_entries.first().unwrap();
    let second_time_entry = time_entries.get(1).unwrap();

    assert_eq!(first_time_entry.id, TimeEntryId::new(123456789));
    assert_eq!(first_time_entry.wid, WorkspaceId::new(1234567));
    assert_eq!(first_time_entry.description, Some("Wurst".to_owned()));

    assert_eq!(second_time_entry.id, TimeEntryId::new(987654321));
    assert_eq!(second_time_entry.wid, WorkspaceId::new(1234567));
    assert_eq!(second_time_entry.description, Some("Kaese".to_owned()));
  }

  mock.assert();

  Ok(())
}

#[test]
fn create_time_entry() -> anyhow::Result<()> {
  let request_body = json!(
    {
      "description": "Wurst",
      "workspace_id": 123456789,
      "tags": ["aa", "bb"],
      "duration": 200,
      "start": "2021-11-21T23:58:09+01:00",
      "project_id": 123456789,
      "created_with": CREATED_WITH,
      "billable": true,
    }
  );

  let response_body = json!(
    {
      "id": 1234567890,
      "wid": 123456789,
      "pid": 123456789,
      "billable": true,
      "start": "2021-11-21T23:58:09+01:00",
      "duration": 200,
      "description": "Wurst",
      "tags": [
        "aa",
        "bb"
      ],
      "duronly": false,
      "at": "2021-11-21T23:58:09+01:00",
      "uid": 123456789
    }
  );

  let mut server = mockito::Server::new();

  let mock = server
    .mock("POST", "/workspaces/123456789/time_entries")
    .with_header(
      "Authorization",
      "Basic Y2I3YmY3ZWZhNmQ2NTIwNDZhYmQyZjdkODRlZTE4YzE6YXBpX3Rva2Vu",
    )
    .with_status(200)
    .match_body(Matcher::Json(request_body))
    .with_body(response_body.to_string())
    .expect(1)
    .create();

  {
    let client = TogglClient::new_with_base_url(
      "cb7bf7efa6d652046abd2f7d84ee18c1".to_owned(),
      server.url().parse()?,
    )?;

    let created_time_entry = client.create_time_entry(
      false,
      Some("Wurst"),
      WorkspaceId::new(123456789),
      Some(vec!["aa".to_owned(), "bb".to_owned()]).as_deref(),
      Duration::try_seconds(200).unwrap(),
      DateTime::<Local>::from_str("2021-11-21T23:58:09+01:00")?,
      ProjectId::new(123456789),
      false,
    )?;

    assert_eq!(
      created_time_entry.start,
      DateTime::<Local>::from_str("2021-11-21T23:58:09+01:00")?
    );
    assert_eq!(
      created_time_entry.tags,
      Some(vec!["aa".to_owned(), "bb".to_owned()])
    );
    assert_eq!(created_time_entry.description, Some("Wurst".to_owned()));
  }

  mock.assert();

  Ok(())
}

#[test]
fn create_client() -> anyhow::Result<()> {
  let request_body = json!(
    {
      "active": true,
      "name": "fkbr.org",
      "wid": 123456789
    }
  );

  let response_body = json!(
    {
      "id": 1234567890,
      "wid": 123456789,
      "name": "fkbr.org",
      "archived": false
    }
  );

  let mut server = mockito::Server::new();

  let mock = server
    .mock("POST", "/workspaces/123456789/clients")
    .with_header(
      "Authorization",
      "Basic Y2I3YmY3ZWZhNmQ2NTIwNDZhYmQyZjdkODRlZTE4YzE6YXBpX3Rva2Vu",
    )
    .with_status(200)
    .match_body(Matcher::Json(request_body))
    .with_body(response_body.to_string())
    .expect(1)
    .create();

  {
    let client = TogglClient::new_with_base_url(
      "cb7bf7efa6d652046abd2f7d84ee18c1".to_owned(),
      server.url().parse()?,
    )?;

    let created_client =
      client.create_client(false, "fkbr.org", WorkspaceId::new(123456789))?;

    assert_eq!(created_client.name, "fkbr.org");
  }

  mock.assert();

  Ok(())
}

#[test]
fn test_start_time_entry() -> anyhow::Result<()> {
  let request_body = json!(
    {
      "at":"2021-11-21T23:58:09+01:00",
      "description": "fkbr",
      "tags": ["a", "b"],
      "pid": 123,
      "start": "2021-11-21T23:58:09+01:00",
      "duration": -1637535489,
      "created_with": CREATED_WITH,
      "billable": false,
      "wid": 123456
    }
  );

  let response_body = json!(
    {
      "id": 123456789,
      "pid": 123,
      "wid": 123456,
      "billable": false,
      "start": "2013-03-05T07:58:58.000Z",
      "duration": -1637535489,
      "description": "fkbr",
      "tags": ["a", "b"]
    }
  );

  let mut server = mockito::Server::new();

  let mock = server
    .mock("POST", "/time_entries")
    .with_header(
      "Authorization",
      "Basic Y2I3YmY3ZWZhNmQ2NTIwNDZhYmQyZjdkODRlZTE4YzE6YXBpX3Rva2Vu",
    )
    .with_status(200)
    .match_body(Matcher::Json(request_body))
    .with_body(response_body.to_string())
    .expect(1)
    .create();

  {
    let client = TogglClient::new_with_base_url(
      "cb7bf7efa6d652046abd2f7d84ee18c1".to_owned(),
      server.url().parse()?,
    )?;

    let started_time_entry = client.start_time_entry(
      false,
      DateTime::<Local>::from_str("2021-11-21T23:58:09+01:00")?,
      WorkspaceId::new(123456),
      Some("fkbr"),
      Some(vec!["a".to_owned(), "b".to_owned()]).as_deref(),
      ProjectId::new(123),
      true,
    )?;

    assert_eq!(started_time_entry.id, TimeEntryId::new(123456789));
  }

  mock.assert();

  Ok(())
}

#[test]
fn test_stop_time_entry() -> anyhow::Result<()> {
  let response_body = json!(
    {
      "id": 123,
      "pid": 123,
      "wid": 456,
      "billable": false,
      "start": "2013-03-05T07:58:58.000Z",
      "duration": 60,
      "description": "fkbr",
      "tags": ["a", "b"]
    }
  );

  let mut server = mockito::Server::new();

  let mock = server
    .mock("PATCH", "/workspaces/456/time_entries/123/stop")
    .with_header(
      "Authorization",
      "Basic Y2I3YmY3ZWZhNmQ2NTIwNDZhYmQyZjdkODRlZTE4YzE6YXBpX3Rva2Vu",
    )
    .with_status(200)
    .with_body(response_body.to_string())
    .expect(1)
    .create();

  {
    let client = TogglClient::new_with_base_url(
      "cb7bf7efa6d652046abd2f7d84ee18c1".to_owned(),
      server.url().parse()?,
    )?;

    let started_time_entry = client.stop_time_entry(
      false,
      WorkspaceId::new(456),
      TimeEntryId::new(123),
    )?;

    assert_eq!(started_time_entry.id, TimeEntryId::new(123));
  }

  mock.assert();

  Ok(())
}

#[test]
fn test_delete_time_entry() -> anyhow::Result<()> {
  let mut server = mockito::Server::new();

  let mock = server
    .mock("DELETE", "/time_entries/456")
    .with_header(
      "Authorization",
      "Basic Y2I3YmY3ZWZhNmQ2NTIwNDZhYmQyZjdkODRlZTE4YzE6YXBpX3Rva2Vu",
    )
    .with_status(200)
    .expect(1)
    .create();

  {
    let client = TogglClient::new_with_base_url(
      "cb7bf7efa6d652046abd2f7d84ee18c1".to_owned(),
      server.url().parse()?,
    )?;

    let deleted_time_entry =
      client.delete_time_entry(false, TimeEntryId::new(456));

    assert_eq!(deleted_time_entry.is_ok(), true);
  }

  mock.assert();

  Ok(())
}
