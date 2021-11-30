use crate::{
  client::{TogglClient, CREATED_WITH},
  model::Range,
};
use chrono::{DateTime, Duration, Local, NaiveDate};
use mockito::{mock, Matcher};
use pretty_assertions::assert_eq;
use serde_json::json;
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
fn get_me() -> anyhow::Result<()> {
  let body = json!(
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
  );

  let mock = mock("GET", "/me")
    .with_header(
      "Authorization",
      "Basic Y2I3YmY3ZWZhNmQ2NTIwNDZhYmQyZjdkODRlZTE4YzE6YXBpX3Rva2Vu",
    )
    .with_status(200)
    .with_body(body.to_string())
    .expect(1)
    .create();

  {
    let client =
      TogglClient::new("cb7bf7efa6d652046abd2f7d84ee18c1".to_string())?;
    let me = client.get_me()?;

    assert_eq!(me.data.email, "ralph.bower@fkbr.org");
    assert_eq!(me.data.api_token, "cb7bf7efa6d652046abd2f7d84ee18c1");
    assert_eq!(me.data.fullname, "Ralph Bower");
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

  let mock = mock("GET", "/workspaces")
    .with_header(
      "Authorization",
      "Basic Y2I3YmY3ZWZhNmQ2NTIwNDZhYmQyZjdkODRlZTE4YzE6YXBpX3Rva2Vu",
    )
    .with_status(200)
    .with_body(body.to_string())
    .expect(1)
    .create();

  {
    let client =
      TogglClient::new("cb7bf7efa6d652046abd2f7d84ee18c1".to_string())?;

    let workspaces = client.get_workspaces()?;
    let first_workspace = workspaces.first().unwrap();

    assert_eq!(first_workspace.id, 1234567);
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
        "at": "2021-11-16T09:30:21+00:00"
      },
      {
        "id": 2345,
        "wid": 12345678,
        "name": "beta male gmbh",
        "at": "2021-11-16T08:42:34+00:00"
      }
    ]
  );

  let mock = mock("GET", "/workspaces/12345678/clients")
    .with_header(
      "Authorization",
      "Basic Y2I3YmY3ZWZhNmQ2NTIwNDZhYmQyZjdkODRlZTE4YzE6YXBpX3Rva2Vu",
    )
    .with_status(200)
    .with_body(body.to_string())
    .expect(1)
    .create();

  {
    let client =
      TogglClient::new("cb7bf7efa6d652046abd2f7d84ee18c1".to_string())?;

    let clients = client.get_workspace_clients(12345678)?;
    let first_client = clients.get(0).unwrap();
    let second_client = clients.get(1).unwrap();

    assert_eq!(first_client.id, 1234);
    assert_eq!(first_client.wid, 12345678);
    assert_eq!(first_client.name, "fkbr.org");

    assert_eq!(second_client.id, 2345);
    assert_eq!(second_client.wid, 12345678);
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
  );

  let mock = mock("GET", "/workspaces/12345678/projects")
    .with_header(
      "Authorization",
      "Basic Y2I3YmY3ZWZhNmQ2NTIwNDZhYmQyZjdkODRlZTE4YzE6YXBpX3Rva2Vu",
    )
    .with_status(200)
    .with_body(body.to_string())
    .expect(1)
    .create();

  {
    let client =
      TogglClient::new("cb7bf7efa6d652046abd2f7d84ee18c1".to_string())?;

    let projects = client.get_workspace_projects(12345678)?;
    let first_project = projects.get(0).unwrap();
    let second_project = projects.get(1).unwrap();

    assert_eq!(first_project.id, 123456789);
    assert_eq!(first_project.wid, 1234567);
    assert_eq!(first_project.name, "beta male gmbh");

    assert_eq!(second_project.id, 987654321);
    assert_eq!(second_project.wid, 1234567);
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

  let mock = mock("GET", "/time_entries?start_date=2021-11-21T00%3A00%3A00%2B01%3A00&end_date=2021-11-22T00%3A00%3A00%2B01%3A00")
      .with_header(
        "Authorization",
        "Basic Y2I3YmY3ZWZhNmQ2NTIwNDZhYmQyZjdkODRlZTE4YzE6YXBpX3Rva2Vu",
      )
      .with_status(200)
      .with_body(body.to_string())
      .expect(1)
      .create();

  {
    let client =
      TogglClient::new("cb7bf7efa6d652046abd2f7d84ee18c1".to_string())?;

    let time_entries = client
      .get_time_entries(&Range::Date(NaiveDate::from_ymd(2021, 11, 21)))?;
    let first_time_entry = time_entries.get(0).unwrap();
    let second_time_entry = time_entries.get(1).unwrap();

    assert_eq!(first_time_entry.id, 123456789);
    assert_eq!(first_time_entry.wid, 1234567);
    assert_eq!(first_time_entry.description, Some("Wurst".to_string()));

    assert_eq!(second_time_entry.id, 987654321);
    assert_eq!(second_time_entry.wid, 1234567);
    assert_eq!(second_time_entry.description, Some("Kaese".to_string()));
  }

  mock.assert();

  Ok(())
}

#[test]
fn create_time_entry() -> anyhow::Result<()> {
  let request_body = json!(
    {
      "time_entry": {
        "description": "Wurst",
        "wid": 123456789,
        "tags": ["aa", "bb"],
        "duration": 200,
        "start": "2021-11-21T23:58:09+01:00",
        "pid": 123456789,
        "created_with": CREATED_WITH,
        "billable": true,
      }
    }
  );

  let response_body = json!(
    {
      "data": {
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
    }
  );

  let mock = mock("POST", "/time_entries")
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
    let client =
      TogglClient::new("cb7bf7efa6d652046abd2f7d84ee18c1".to_string())?;

    let created_time_entry = client.create_time_entry(
      "Wurst",
      123456789,
      &Some(vec!["aa".to_string(), "bb".to_string()]),
      Duration::seconds(200),
      DateTime::<Local>::from_str("2021-11-21T23:58:09+01:00")?,
      123456789,
      false,
    )?;

    assert_eq!(
      created_time_entry.data.start,
      DateTime::<Local>::from_str("2021-11-21T23:58:09+01:00")?
    );
    assert_eq!(created_time_entry.data.tags, vec!["aa", "bb"]);
    assert_eq!(
      created_time_entry.data.description,
      Some("Wurst".to_string())
    );
  }

  mock.assert();

  Ok(())
}

#[test]
fn create_client() -> anyhow::Result<()> {
  let request_body = json!(
    {
      "client": {
        "name": "fkbr.org",
        "wid": 123456789
      }
    }
  );

  let response_body = json!(
    {
      "data": {
        "id": 1234567890,
        "wid": 123456789,
        "name": "fkbr.org"
      }
    }
  );

  let mock = mock("POST", "/clients")
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
    let client =
      TogglClient::new("cb7bf7efa6d652046abd2f7d84ee18c1".to_string())?;

    let created_client = client.create_client("fkbr.org", 123456789)?;

    assert_eq!(created_client.data.name, "fkbr.org");
  }

  mock.assert();

  Ok(())
}

#[test]
fn test_start_time_entry() -> anyhow::Result<()> {
  let request_body = json!(
    {
      "time_entry": {
        "description": "fkbr",
        "tags": ["a", "b"],
        "pid": 123,
        "created_with": CREATED_WITH,
        "billable": false,
      }
    }
  );

  let response_body = json!(
    {
      "data": {
        "id": 123456789,
        "pid": 123,
        "wid": 123456,
        "billable": false,
        "start": "2013-03-05T07:58:58.000Z",
        "duration": -1362470338,
        "description": "fkbr",
        "tags": ["a", "b"]
      }
    }
  );

  let mock = mock("POST", "/time_entries/start")
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
    let client =
      TogglClient::new("cb7bf7efa6d652046abd2f7d84ee18c1".to_string())?;

    let started_time_entry = client.start_time_entry(
      "fkbr",
      &Some(vec!["a".to_string(), "b".to_string()]),
      123,
      true,
    )?;

    assert_eq!(started_time_entry.data.id, 123456789);
  }

  mock.assert();

  Ok(())
}

#[test]
fn test_stop_time_entry() -> anyhow::Result<()> {
  let request_body = json!(
    {
      "time_entry": {
        "description": "fkbr",
        "tags": ["a", "b"],
        "pid": 123,
        "created_with": CREATED_WITH,
      }
    }
  );

  let response_body = json!(
    {
      "data": {
        "id": 123456789,
        "pid": 123,
        "wid": 123456,
        "billable": false,
        "start": "2013-03-05T07:58:58.000Z",
        "duration": 60,
        "description": "fkbr",
        "tags": ["a", "b"]
      }
    }
  );

  let mock = mock("PUT", "/time_entries/456/stop")
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
    let client =
      TogglClient::new("cb7bf7efa6d652046abd2f7d84ee18c1".to_string())?;

    let started_time_entry = client.stop_time_entry(
      456,
      "fkbr",
      &Some(vec!["a".to_string(), "b".to_string()]),
      123,
    )?;

    assert_eq!(started_time_entry.data.id, 123456789);
  }

  mock.assert();

  Ok(())
}
