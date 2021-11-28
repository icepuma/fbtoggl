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
      start: Start::Date(DateTime::<Local>::from_str("2021-11-21T22:58:09Z")?),
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
