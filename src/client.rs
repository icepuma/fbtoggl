use std::fmt::Debug;

use crate::config::read_settings;
use crate::model::Client;
use crate::model::Me;
use crate::model::Project;
use crate::model::Range;
use crate::model::TimeEntry;
use crate::model::Workspace;
use anyhow::anyhow;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::DateTime;
use chrono::Duration;
use chrono::Local;
use colored::Colorize;
use minreq::Method;
use minreq::Request;
use minreq::Response;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::json;
use url::Url;

pub struct TogglClient {
  base_url: Url,
  api_token: String,
}

pub const CREATED_WITH: &str = "fbtoggl (https://github.com/icepuma/fbtoggl)";

const AUTHORIZATION: &str = "Authorization";

pub fn init_client() -> anyhow::Result<TogglClient> {
  let settings = read_settings()?;

  TogglClient::new(settings.api_token)
}

impl TogglClient {
  pub fn new(api_token: String) -> anyhow::Result<TogglClient> {
    let base_url = "https://api.track.toggl.com/api/v9/".parse()?;

    Ok(TogglClient {
      base_url,
      api_token,
    })
  }

  #[cfg(test)]
  pub fn new_with_base_url(
    api_token: String,
    base_url: Url,
  ) -> anyhow::Result<TogglClient> {
    Ok(TogglClient {
      base_url,
      api_token,
    })
  }

  fn basic_auth(&self) -> (String, String) {
    (
      AUTHORIZATION.to_string(),
      format!(
        "Basic {}",
        STANDARD.encode(format!("{}:api_token", &self.api_token))
      ),
    )
  }

  fn base_request(&self, method: Method, uri: &str) -> anyhow::Result<Request> {
    let url = self.base_url.join(uri)?;

    let (key, value) = self.basic_auth();

    Ok(minreq::Request::new(method, url).with_header(key, value))
  }

  fn request<D: DeserializeOwned + Debug>(
    &self,
    debug: bool,
    method: Method,
    uri: &str,
  ) -> anyhow::Result<D> {
    let request = self.base_request(method, uri)?;

    if debug {
      println!("{}", "Request:".bold().underline());
      println!("{request:?}");
      println!();
    }

    let response = request.send()?;

    self.response(debug, response)
  }

  fn empty_request(
    &self,
    debug: bool,
    method: Method,
    uri: &str,
  ) -> anyhow::Result<()> {
    let request = self.base_request(method, uri)?;

    if debug {
      println!("{}", "Request:".bold().underline());
      println!("{request:?}");
      println!();
    }

    let response = request.send()?;

    self.empty_response(response)
  }

  fn request_with_body<D: DeserializeOwned + Debug, S: Serialize + Debug>(
    &self,
    debug: bool,
    method: Method,
    uri: &str,
    body: S,
  ) -> anyhow::Result<D> {
    let request = self.base_request(method, uri)?.with_json(&body)?;

    if debug {
      println!("{}", "Request:".bold().underline());
      println!("{request:?}");
      println!();
      println!("{:?}", &body);
      println!();
    }

    let response = request.send()?;

    self.response(debug, response)
  }

  fn response<D: DeserializeOwned + Debug>(
    &self,
    debug: bool,
    response: Response,
  ) -> anyhow::Result<D> {
    if debug {
      println!("{}", "Response:".bold().underline());
      println!("{response:?}");
      println!();
    }

    match response.status_code {
      200 | 201 if debug => match response.json() {
        Ok(json) => {
          println!("{}", "Received JSON response:".bold().underline());
          println!("{json:?}");
          println!();

          Ok(json)
        }
        Err(err) => Err(anyhow!("Failed to deserialize JSON: {}", err)),
      },
      200 | 201 => Ok(response.json()?),
      status => match response.as_str() {
        Ok(text) => Err(anyhow!("{} - {}", status, text)),
        Err(_) => Err(anyhow!("{}", status)),
      },
    }
  }

  fn empty_response(&self, response: Response) -> anyhow::Result<()> {
    match response.status_code {
      200 | 201 => Ok(()),
      status => match response.as_str() {
        Ok(text) => Err(anyhow!("{} - {}", status, text)),
        Err(_) => Err(anyhow!("{}", status)),
      },
    }
  }

  pub fn get_workspace_clients(
    &self,
    debug: bool,
    include_archived: bool,
    workspace_id: u64,
  ) -> anyhow::Result<Option<Vec<Client>>> {
    let uri = if include_archived {
      format!("workspaces/{workspace_id}/clients?status=both")
    } else {
      format!("workspaces/{workspace_id}/clients?status=active")
    };

    self.request(debug, Method::Get, &uri)
  }

  pub fn get_time_entries(
    &self,
    debug: bool,
    range: &Range,
  ) -> anyhow::Result<Vec<TimeEntry>> {
    let (start, end) = range.as_range()?;
    let start_date = start.format("%Y-%m-%d").to_string();

    // End date is not inclusive, therefore we add one day
    let end_date = (end + Duration::try_days(1).unwrap())
      .format("%Y-%m-%d")
      .to_string();

    let uri = format!(
      "me/time_entries?start_date={}&end_date={}",
      urlencoding::encode(&start_date),
      urlencoding::encode(&end_date),
    );

    self.request::<Vec<TimeEntry>>(debug, Method::Get, &uri)
  }

  pub fn get_workspaces(&self, debug: bool) -> anyhow::Result<Vec<Workspace>> {
    self.request::<Vec<Workspace>>(debug, Method::Get, "workspaces")
  }

  pub fn get_me(&self, debug: bool) -> anyhow::Result<Me> {
    self.request::<Me>(debug, Method::Get, "me")
  }

  pub fn get_workspace_projects(
    &self,
    debug: bool,
    include_archived: bool,
    workspace_id: u64,
  ) -> anyhow::Result<Vec<Project>> {
    let uri = if include_archived {
      format!("workspaces/{workspace_id}/projects")
    } else {
      format!("workspaces/{workspace_id}/projects?active=true")
    };

    self.request::<Vec<Project>>(debug, Method::Get, &uri)
  }

  #[allow(clippy::too_many_arguments)]
  pub fn create_time_entry(
    &self,
    debug: bool,
    description: &Option<String>,
    workspace_id: u64,
    tags: &Option<Vec<String>>,
    duration: Duration,
    start: DateTime<Local>,
    project_id: u64,
    non_billable: bool,
  ) -> anyhow::Result<TimeEntry> {
    let billable = !non_billable;

    let body = json!({
      "description": description,
      "workspace_id": workspace_id,
      "tags": tags,
      "duration": duration.num_seconds(),
      "start": start,
      "project_id": project_id,
      "created_with": CREATED_WITH,
      "billable": billable,
    });

    let uri = format!("workspaces/{workspace_id}/time_entries");

    self.request_with_body(debug, Method::Post, &uri, body)
  }

  pub fn create_client(
    &self,
    debug: bool,
    name: &str,
    workspace_id: u64,
  ) -> anyhow::Result<Client> {
    let body = json!({
      "active": true,
      "name": name,
      "wid": workspace_id,
    });

    let uri = format!("workspaces/{workspace_id}/clients");

    self.request_with_body(debug, Method::Post, &uri, body)
  }

  #[allow(clippy::too_many_arguments)]
  pub fn start_time_entry(
    &self,
    debug: bool,
    start: DateTime<Local>,
    workspace_id: u64,
    description: &Option<String>,
    tags: &Option<Vec<String>>,
    project_id: u64,
    non_billable: bool,
  ) -> anyhow::Result<TimeEntry> {
    let billable = !non_billable;
    let duration = -start.timestamp();

    let body = json!({
      "at": start,
      "billable": billable,
      "created_with": CREATED_WITH,
      "description": description,
      "duration": duration,
      "pid": project_id,
      "start": start,
      "tags": tags,
      "wid": workspace_id
    });

    let uri = "time_entries".to_string();

    self.request_with_body(debug, Method::Post, &uri, body)
  }

  pub fn stop_time_entry(
    &self,
    debug: bool,
    workspace_id: u64,
    time_entry_id: u64,
  ) -> anyhow::Result<TimeEntry> {
    self.request(
      debug,
      Method::Patch,
      &format!("workspaces/{workspace_id}/time_entries/{time_entry_id}/stop"),
    )
  }

  pub fn delete_time_entry(
    &self,
    debug: bool,
    time_entry_id: u64,
  ) -> anyhow::Result<()> {
    self.empty_request(
      debug,
      Method::Delete,
      &format!("time_entries/{time_entry_id}"),
    )
  }
}
