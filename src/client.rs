use anyhow::anyhow;
use chrono::DateTime;
use chrono::Utc;
use reqwest::blocking;
use reqwest::Method;
use reqwest::StatusCode;
use reqwest::Url;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::json;

use crate::model::Client;
use crate::model::DataWith;
use crate::model::Project;
use crate::model::SinceWith;
use crate::model::TimeEntry;
use crate::model::UserData;
use crate::model::Workspace;

#[cfg(test)]
use mockito;

pub struct TogglClient {
  base_url: Url,
  client: blocking::Client,
  api_token: String,
}

const CREATED_WITH: &str = "fbtoggl (https://github.com/icepuma/fbtoggl)";

impl TogglClient {
  pub fn new(api_token: String) -> anyhow::Result<TogglClient> {
    #[cfg(not(test))]
    let base_url = "https://api.track.toggl.com/api/v8/".parse()?;

    #[cfg(test)]
    let base_url = mockito::server_url().parse()?;

    let client = blocking::Client::new();

    Ok(TogglClient {
      base_url,
      client,
      api_token,
    })
  }

  fn base_request(
    &self,
    method: Method,
    uri: &str,
  ) -> anyhow::Result<blocking::RequestBuilder> {
    let url = self.base_url.join(uri)?;

    Ok(
      self
        .client
        .request(method, url)
        .basic_auth(&self.api_token, Some("api_token")),
    )
  }

  fn request<D: DeserializeOwned>(
    &self,
    method: Method,
    uri: &str,
  ) -> anyhow::Result<D> {
    let response = self.base_request(method, uri)?.send()?;

    self.response(response)
  }

  fn request_with_body<D: DeserializeOwned, S: Serialize>(
    &self,
    method: Method,
    uri: &str,
    body: S,
  ) -> anyhow::Result<D> {
    let response = self.base_request(method, uri)?.json(&body).send()?;

    self.response(response)
  }

  fn response<D: DeserializeOwned>(
    &self,
    response: blocking::Response,
  ) -> anyhow::Result<D> {
    match response.status() {
      StatusCode::OK | StatusCode::CREATED => Ok(response.json()?),
      status => Err(anyhow!("Error: {}", status)),
    }
  }

  pub fn get_workspace_clients(
    &self,
    workspace_id: u64,
  ) -> anyhow::Result<Vec<Client>> {
    self.request::<Vec<Client>>(
      Method::GET,
      &format!("workspaces/{}/clients", workspace_id),
    )
  }

  pub fn get_time_entries(&self) -> anyhow::Result<Vec<TimeEntry>> {
    self.request::<Vec<TimeEntry>>(Method::GET, "time_entries")
  }

  pub fn get_workspaces(&self) -> anyhow::Result<Vec<Workspace>> {
    self.request::<Vec<Workspace>>(Method::GET, "workspaces")
  }

  pub fn get_me(&self) -> anyhow::Result<SinceWith<UserData>> {
    self.request::<SinceWith<UserData>>(Method::GET, "me")
  }

  pub fn get_workspace_projects(
    &self,
    workspace_id: u64,
  ) -> anyhow::Result<Vec<Project>> {
    self.request::<Vec<Project>>(
      Method::GET,
      &format!("workspaces/{}/projects", workspace_id),
    )
  }

  pub fn create_time_entry(
    &self,
    description: &str,
    workspace_id: u64,
    tags: &Option<Vec<String>>,
    duration: u64,
    start: DateTime<Utc>,
    project_id: u64,
  ) -> anyhow::Result<DataWith<TimeEntry>> {
    let body = json!({
        "time_entry": {
            "description": description,
            "wid": workspace_id,
            "tags": tags,
            "duration": duration,
            "start": start,
            "pid": project_id,
            "created_with": CREATED_WITH,
        }
    });

    self.request_with_body(Method::POST, "time_entries", body)
  }

  pub fn create_client(
    &self,
    name: &str,
    workspace_id: u64,
  ) -> anyhow::Result<DataWith<Client>> {
    let body = json!({
        "client": {
            "name": name,
            "wid": workspace_id
        }
    });

    self.request_with_body(Method::POST, "clients", body)
  }
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;

  use chrono::{DateTime, Utc};
  use mockito::{mock, Matcher};
  use serde_json::{json, Value};

  use crate::client::{TogglClient, CREATED_WITH};
  use pretty_assertions::assert_eq;

  #[ctor::ctor]
  fn setup() {
    std::env::set_var("RUST_LOG", "mockito=debug");

    let _ = env_logger::try_init();
  }

  #[test]
  fn get_me() -> anyhow::Result<()> {
    let body = me();

    let mock = mock("GET", "/me")
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

    let mock = mock("GET", "/time_entries")
      .with_status(200)
      .with_body(body.to_string())
      .expect(1)
      .create();

    {
      let client =
        TogglClient::new("cb7bf7efa6d652046abd2f7d84ee18c1".to_string())?;

      let time_entries = client.get_time_entries()?;
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
          "start": "2021-11-21T22:58:09Z",
          "pid": 123456789,
          "created_with": CREATED_WITH,
        }
      }
    );

    let response_body = json!(
      {
        "data": {
          "id": 1234567890,
          "wid": 123456789,
          "pid": 123456789,
          "billable": false,
          "start": "2021-11-21T22:58:09Z",
          "duration": 200,
          "description": "Wurst",
          "tags": [
            "aa",
            "bb"
          ],
          "duronly": false,
          "at": "2021-11-21T22:58:09Z",
          "uid": 123456789
        }
      }
    );

    let create_time_entry_mock = mock("POST", "/time_entries")
      .with_status(200)
      .match_body(Matcher::Json(request_body))
      .with_body(response_body.to_string())
      .expect(1)
      .create();

    {
      let client =
        TogglClient::new("cb7bf7efa6d652046abd2f7d84ee18c1".to_string())?;

      let create_time_entry = client.create_time_entry(
        "Wurst",
        123456789,
        &Some(vec!["aa".to_string(), "bb".to_string()]),
        200,
        DateTime::<Utc>::from_str("2021-11-21T22:58:09Z")?,
        123456789,
      )?;

      assert_eq!(
        create_time_entry.data.start,
        DateTime::<Utc>::from_str("2021-11-21T22:58:09Z")?
      );
      assert_eq!(create_time_entry.data.tags, vec!["aa", "bb"]);
      assert_eq!(
        create_time_entry.data.description,
        Some("Wurst".to_string())
      );
    }

    create_time_entry_mock.assert();

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
}
