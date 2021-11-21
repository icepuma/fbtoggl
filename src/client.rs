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

pub struct TogglClient {
  base_url: Url,
  client: blocking::Client,
  api_token: String,
}

const TOGGL_BASE_URL: &str = "https://api.track.toggl.com/api/v8/";
const CREATED_WITH: &str = "fbtoggl (https://github.com/icepuma/fbtoggl)";

impl TogglClient {
  pub fn new(api_token: String) -> anyhow::Result<TogglClient> {
    let base_url = TOGGL_BASE_URL.to_string().parse()?;
    let client = blocking::Client::new();

    Ok(TogglClient { base_url, client, api_token })
  }

  fn base_request(&self, method: Method, uri: &str) -> anyhow::Result<blocking::RequestBuilder> {
    let url = self.base_url.join(uri)?;

    Ok(self.client.request(method, url).basic_auth(&self.api_token, Some("api_token")))
  }

  fn request<D: DeserializeOwned>(&self, method: Method, uri: &str) -> anyhow::Result<D> {
    let response = self.base_request(method, uri)?.send()?;

    self.response(response)
  }

  fn request_with_body<D: DeserializeOwned, S: Serialize>(&self, method: Method, uri: &str, body: S) -> anyhow::Result<D> {
    let response = self.base_request(method, uri)?.json(&body).send()?;

    self.response(response)
  }

  fn response<D: DeserializeOwned>(&self, response: blocking::Response) -> anyhow::Result<D> {
    match response.status() {
      StatusCode::OK | StatusCode::CREATED => Ok(response.json()?),
      status => Err(anyhow!("Error: {}", status)),
    }
  }

  pub fn get_workspace_clients(&self, workspace_id: u64) -> anyhow::Result<Vec<Client>> {
    self.request::<Vec<Client>>(Method::GET, &format!("workspaces/{}/clients", workspace_id))
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

  pub fn get_workspace_projects(&self, workspace_id: u64) -> anyhow::Result<Vec<Project>> {
    self.request::<Vec<Project>>(Method::GET, &format!("workspaces/{}/projects", workspace_id))
  }

  pub fn create_time_entry(
    &self,
    description: &str,
    tags: &Option<Vec<String>>,
    duration: u64,
    start: DateTime<Utc>,
    project_id: u64,
  ) -> anyhow::Result<DataWith<TimeEntry>> {
    let body = json!({
        "time_entry": {
            "description": description,
            "tags": tags,
            "duration": duration,
            "start": start,
            "pid": project_id,
            "created_with": CREATED_WITH,
        }
    });

    self.request_with_body(Method::POST, "time_entries", body)
  }

  pub fn create_client(&self, name: &str, workspace_id: u64) -> anyhow::Result<DataWith<Client>> {
    let body = json!({
        "client": {
            "name": name,
            "wid": workspace_id
        }
    });

    self.request_with_body(Method::POST, "clients", body)
  }
}
