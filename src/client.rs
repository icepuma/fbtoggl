use crate::config::read_settings;
use crate::model::Client;
use crate::model::DataWith;
use crate::model::Project;
use crate::model::Range;
use crate::model::SinceWith;
use crate::model::TimeEntry;
use crate::model::UserData;
use crate::model::Workspace;
use anyhow::anyhow;
use chrono::DateTime;
use chrono::Duration;
use chrono::Local;
use reqwest::blocking;
use reqwest::Method;
use reqwest::StatusCode;
use reqwest::Url;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::json;

pub struct TogglClient {
  base_url: Url,
  client: blocking::Client,
  api_token: String,
}

pub const CREATED_WITH: &str = "fbtoggl (https://github.com/icepuma/fbtoggl)";

pub fn init_client() -> anyhow::Result<TogglClient> {
  let settings = read_settings()?;

  TogglClient::new(settings.api_token)
}

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

  fn empty_request(&self, method: Method, uri: &str) -> anyhow::Result<()> {
    let response = self.base_request(method, uri)?.send()?;

    self.empty_response(response)
  }

  fn empty_request_with_body<D: DeserializeOwned>(
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
      status => Err(anyhow!("{}", status)),
    }
  }

  fn empty_response(&self, response: blocking::Response) -> anyhow::Result<()> {
    match response.status() {
      StatusCode::OK | StatusCode::CREATED => Ok(()),
      status => Err(anyhow!("{}", status)),
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

  pub fn get_time_entries(
    &self,
    range: &Range,
  ) -> anyhow::Result<Vec<TimeEntry>> {
    let (start, end) = range.as_range();
    let start_date = start.format("%Y-%m-%dT%H:%M:%S%:z").to_string();
    let end_date = end.format("%Y-%m-%dT%H:%M:%S%:z").to_string();

    let uri = format!(
      "time_entries?start_date={}&end_date={}",
      urlencoding::encode(&start_date),
      urlencoding::encode(&end_date),
    );

    self.request::<Vec<TimeEntry>>(Method::GET, &uri)
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

  #[allow(clippy::too_many_arguments)]
  pub fn create_time_entry(
    &self,
    description: &Option<String>,
    workspace_id: u64,
    tags: &Option<Vec<String>>,
    duration: Duration,
    start: DateTime<Local>,
    project_id: u64,
    non_billable: bool,
  ) -> anyhow::Result<DataWith<TimeEntry>> {
    let billable = !non_billable;

    let body = json!({
        "time_entry": {
            "description": description,
            "wid": workspace_id,
            "tags": tags,
            "duration": duration.num_seconds(),
            "start": start,
            "pid": project_id,
            "created_with": CREATED_WITH,
            "billable": billable,
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

  pub fn start_time_entry(
    &self,
    description: &Option<String>,
    tags: &Option<Vec<String>>,
    project_id: u64,
    non_billable: bool,
  ) -> anyhow::Result<DataWith<TimeEntry>> {
    let billable = !non_billable;

    let body = json!({
      "time_entry": {
        "description": description,
        "tags": tags,
        "pid": project_id,
        "created_with": CREATED_WITH,
        "billable": billable,
      }
    });

    self.request_with_body(Method::POST, "time_entries/start", body)
  }

  pub fn stop_time_entry(
    &self,
    time_entry_id: u64,
    description: &Option<String>,
    tags: &Option<Vec<String>>,
    project_id: u64,
  ) -> anyhow::Result<DataWith<TimeEntry>> {
    let body = json!({
      "time_entry": {
        "description": description,
        "tags": tags,
        "pid": project_id,
        "created_with": CREATED_WITH,
      }
    });

    self.request_with_body(
      Method::PUT,
      &format!("time_entries/{}/stop", time_entry_id),
      body,
    )
  }

  pub fn delete_time_entry(&self, time_entry_id: u64) -> anyhow::Result<()> {
    self
      .empty_request(Method::DELETE, &format!("time_entries/{}", time_entry_id))
  }

  pub fn time_entry_details(
    &self,
    time_entry_id: u64,
  ) -> anyhow::Result<DataWith<TimeEntry>> {
    self.empty_request_with_body(
      Method::GET,
      &format!("time_entries/{}", time_entry_id),
    )
  }
}
