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

    let me = client.get_me()?;

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

    let workspaces = client.get_workspaces()?;
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
      .get_workspace_clients(false, WorkspaceId::new(12345678))?
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
        "workspace_id": 1234567,
        "client_id": 87654321,
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
        "workspace_id": 1234567,
        "client_id": 12345678,
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

    let projects =
      client.get_workspace_projects(false, WorkspaceId::new(12345678))?;
    let first_project = projects.first().unwrap();
    let second_project = projects.get(1).unwrap();

    assert_eq!(first_project.id, ProjectId::new(123456789));
    assert_eq!(first_project.workspace_id, WorkspaceId::new(1234567));
    assert_eq!(first_project.name, "beta male gmbh");

    assert_eq!(second_project.id, ProjectId::new(987654321));
    assert_eq!(second_project.workspace_id, WorkspaceId::new(1234567));
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
        "workspace_id": 1234567,
        "project_id": 123456789,
        "user_id": 1234567,
        "billable": true,
        "start": "2021-11-21T22:57:33+00:00",
        "stop": "2021-11-21T22:57:37+00:00",
        "duration": 4,
        "description": "Wurst",
        "duronly": false,
        "at": "2021-11-21T22:57:37+00:00"
      },
      {
        "id": 987654321,
        "workspace_id": 1234567,
        "project_id": 123456789,
        "user_id": 1234567,
        "billable": true,
        "start": "2021-11-21T22:58:09+00:00",
        "stop": "2021-11-21T22:58:12+00:00",
        "duration": 3,
        "description": "Kaese",
        "duronly": false,
        "at": "2021-11-21T22:58:12+00:00"
      }
    ]
  );

  let mut server = mockito::Server::new();

  let mock = server
    .mock(
      "GET",
      "/me/time_entries?start_date=2021-11-21&end_date=2021-11-22",
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

    let time_entries = client.get_time_entries(&Range::Date(
      NaiveDate::from_ymd_opt(2021, 11, 21).unwrap(),
    ))?;
    let first_time_entry = time_entries.first().unwrap();
    let second_time_entry = time_entries.get(1).unwrap();

    assert_eq!(first_time_entry.id, TimeEntryId::new(123456789));
    assert_eq!(first_time_entry.workspace_id, WorkspaceId::new(1234567));
    assert_eq!(first_time_entry.description, Some("Wurst".to_owned()));

    assert_eq!(second_time_entry.id, TimeEntryId::new(987654321));
    assert_eq!(second_time_entry.workspace_id, WorkspaceId::new(1234567));
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
      "start": "2021-11-21T22:58:09Z",
      "project_id": 123456789,
      "created_with": CREATED_WITH,
      "billable": true,
    }
  );

  let response_body = json!(
    {
      "id": 1234567890,
      "workspace_id": 123456789,
      "project_id": 123456789,
      "user_id": 123456789,
      "billable": true,
      "start": "2021-11-21T23:58:09+01:00",
      "duration": 200,
      "description": "Wurst",
      "tags": [
        "aa",
        "bb"
      ],
      "duronly": false,
      "at": "2021-11-21T23:58:09+01:00"
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
      "name": "fkbr.org",
      "workspace_id": 123456789,
    }
  );

  let response_body = json!(
    {
      "id": 1234567890,
      "workspace_id": 123456789,
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
      client.create_client("fkbr.org", WorkspaceId::new(123456789))?;

    assert_eq!(created_client.name, "fkbr.org");
  }

  mock.assert();

  Ok(())
}

#[test]
fn test_start_time_entry() -> anyhow::Result<()> {
  let request_body = json!(
    {
      "description": "fkbr",
      "tags": ["a", "b"],
      "project_id": 123,
      "start": "2021-11-21T22:58:09Z",
      "duration": -1,
      "created_with": CREATED_WITH,
      "billable": false,
      "workspace_id": 123456,
    }
  );

  let response_body = json!(
    {
      "id": 123456789,
      "project_id": 123,
      "workspace_id": 123456,
      "billable": false,
      "start": "2013-03-05T07:58:58.000Z",
      "duration": -1,
      "description": "fkbr",
      "tags": ["a", "b"]
    }
  );

  let mut server = mockito::Server::new();

  let mock = server
    .mock("POST", "/workspaces/123456/time_entries")
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
      "project_id": 123,
      "workspace_id": 456,
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

    let started_time_entry =
      client.stop_time_entry(WorkspaceId::new(456), TimeEntryId::new(123))?;

    assert_eq!(started_time_entry.id, TimeEntryId::new(123));
  }

  mock.assert();

  Ok(())
}

#[test]
fn test_delete_time_entry() -> anyhow::Result<()> {
  let mut server = mockito::Server::new();

  let mock = server
    .mock("DELETE", "/workspaces/789/time_entries/456")
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
      client.delete_time_entry(WorkspaceId::new(789), TimeEntryId::new(456));

    assert_eq!(deleted_time_entry.is_ok(), true);
  }

  mock.assert();

  Ok(())
}

#[test]
fn get_workspace_clients_include_archived_uses_status_both()
-> anyhow::Result<()> {
  let body = json!([]);

  let mut server = mockito::Server::new();

  let mock = server
    .mock("GET", "/workspaces/42/clients?status=both")
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

    client.get_workspace_clients(true, WorkspaceId::new(42))?;
  }

  mock.assert();

  Ok(())
}

#[test]
fn get_workspace_projects_include_archived_drops_active_filter()
-> anyhow::Result<()> {
  let body = json!([]);

  let mut server = mockito::Server::new();

  let mock = server
    .mock("GET", "/workspaces/42/projects")
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

    client.get_workspace_projects(true, WorkspaceId::new(42))?;
  }

  mock.assert();

  Ok(())
}

#[test]
fn get_time_entries_from_to_range_uses_exclusive_end() -> anyhow::Result<()> {
  // Range::FromTo(2021-11-01, 2021-11-03) covers entries on Nov 1, 2, and 3.
  // The Toggl v9 end_date is exclusive, so the URL end_date is Nov 4.
  // Verifies bug #1 from the audit stays fixed (the +1 day must not be
  // applied twice).
  let body = json!([]);

  let mut server = mockito::Server::new();

  let mock = server
    .mock(
      "GET",
      "/me/time_entries?start_date=2021-11-01&end_date=2021-11-04",
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

    client.get_time_entries(&Range::FromTo(
      NaiveDate::from_ymd_opt(2021, 11, 1).unwrap(),
      NaiveDate::from_ymd_opt(2021, 11, 3).unwrap(),
    ))?;
  }

  mock.assert();

  Ok(())
}

#[test]
fn update_time_entry_omits_stop_when_running() -> anyhow::Result<()> {
  use crate::model::TimeEntry;
  use chrono::Utc;
  use mockito::Matcher;

  // For a running entry (stop=None, duration=-1) we must NOT send `stop: null`
  // in the PUT body — Toggl v9 rejects that combo as inconsistent.
  let running = TimeEntry {
    id: TimeEntryId::new(99),
    workspace_id: WorkspaceId::new(7),
    project_id: Some(ProjectId::new(1)),
    billable: Some(true),
    start: DateTime::<Utc>::from_str("2021-11-21T22:57:33+00:00")?,
    stop: None,
    duration: -1,
    description: Some("running".to_owned()),
    tags: None,
    duronly: false,
  };

  let response = json!({
    "id": 99,
    "workspace_id": 7,
    "project_id": 1,
    "billable": true,
    "start": "2021-11-21T22:57:33+00:00",
    "duration": -1,
    "description": "running"
  });

  let mut server = mockito::Server::new();

  // Exact body match: if the code mistakenly adds "stop": null this fails.
  let expected_body = json!({
    "billable": true,
    "description": "running",
    "duration": -1,
    "project_id": 1,
    "start": "2021-11-21T22:57:33Z",
    "tags": null,
    "workspace_id": 7,
  });

  let mock = server
    .mock("PUT", "/workspaces/7/time_entries/99")
    .with_header(
      "Authorization",
      "Basic Y2I3YmY3ZWZhNmQ2NTIwNDZhYmQyZjdkODRlZTE4YzE6YXBpX3Rva2Vu",
    )
    .with_status(200)
    .match_body(Matcher::Json(expected_body))
    .with_body(response.to_string())
    .expect(1)
    .create();

  {
    let client = TogglClient::new_with_base_url(
      "cb7bf7efa6d652046abd2f7d84ee18c1".to_owned(),
      server.url().parse()?,
    )?;

    client.update_time_entry(TimeEntryId::new(99), &running)?;
  }

  mock.assert();

  Ok(())
}
