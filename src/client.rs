pub use crate::common::CREATED_WITH;
use crate::config::read_settings;
use crate::error::Result;
use crate::http_client::{HttpClient, HttpClientExt};
use crate::model::Client;
use crate::model::Me;
use crate::model::Project;
use crate::model::Range;
use crate::model::Workspace;
use crate::model::TimeEntry;
use crate::types::{
  ApiToken, ClientStatusFilter, ProjectId, TimeEntryId, WorkspaceId,
};
use anyhow::Context;
use chrono::DateTime;
use chrono::Duration;
use chrono::Local;
use chrono::Utc;
use minreq::Method;
use serde_json::json;
use url::Url;

pub struct TogglClient {
  base_url: Url,
  api_token: ApiToken,
  debug: bool,
}

pub fn init_client(debug: bool) -> anyhow::Result<TogglClient> {
  let settings =
    read_settings().context("Failed to read configuration settings")?;

  TogglClient::new(settings.api_token, debug)
    .context("Failed to initialize Toggl client")
}

impl TogglClient {
  pub fn new(api_token: ApiToken, debug: bool) -> anyhow::Result<Self> {
    let base_url = "https://api.track.toggl.com/api/v9/"
      .parse()
      .context("Failed to parse Toggl API base URL")?;

    Ok(Self {
      base_url,
      api_token,
      debug,
    })
  }

  #[cfg(test)]
  pub fn new_with_base_url(
    api_token: String,
    base_url: Url,
  ) -> anyhow::Result<Self> {
    let api_token = ApiToken::new(api_token)?;
    Ok(Self {
      base_url,
      api_token,
      debug: false,
    })
  }
}

impl HttpClient for TogglClient {
  fn base_url(&self) -> &Url {
    &self.base_url
  }

  fn api_token(&self) -> &ApiToken {
    &self.api_token
  }

  fn service_name(&self) -> &'static str {
    "Toggl"
  }

  fn debug(&self) -> bool {
    self.debug
  }
}

impl TogglClient {
  pub fn get_workspace_clients(
    &self,
    include_archived: bool,
    workspace_id: WorkspaceId,
  ) -> Result<Option<Vec<Client>>> {
    let status = if include_archived {
      ClientStatusFilter::Both
    } else {
      ClientStatusFilter::Active
    };

    let uri = format!(
      "workspaces/{}/clients?status={}",
      workspace_id,
      status.as_str()
    );

    self.request(Method::Get, &uri)
  }

  pub fn get_time_entries(&self, range: &Range) -> Result<Vec<TimeEntry>> {
    let (start, end) = range.as_range()?;
    let start_date = start.format("%Y-%m-%d").to_string();
    let end_date = end.format("%Y-%m-%d").to_string();

    let uri = format!(
      "me/time_entries?start_date={}&end_date={}",
      urlencoding::encode(&start_date),
      urlencoding::encode(&end_date),
    );

    self.request::<Vec<TimeEntry>>(Method::Get, &uri)
  }

  pub fn get_workspaces(&self) -> Result<Vec<Workspace>> {
    self.request::<Vec<Workspace>>(Method::Get, "workspaces")
  }

  pub fn get_me(&self) -> Result<Me> {
    self.request::<Me>(Method::Get, "me")
  }

  pub fn get_workspace_projects(
    &self,
    include_archived: bool,
    workspace_id: WorkspaceId,
  ) -> Result<Vec<Project>> {
    let uri = if include_archived {
      format!("workspaces/{workspace_id}/projects")
    } else {
      format!("workspaces/{workspace_id}/projects?active=true")
    };

    self.request::<Vec<Project>>(Method::Get, &uri)
  }

  #[allow(
    clippy::too_many_arguments,
    reason = "All parameters are necessary for the Toggl API"
  )]
  pub fn create_time_entry(
    &self,
    description: Option<&str>,
    workspace_id: WorkspaceId,
    tags: Option<&[String]>,
    duration: Duration,
    start: DateTime<Local>,
    project_id: ProjectId,
    non_billable: bool,
  ) -> Result<TimeEntry> {
    let body = build_time_entry_body(
      description,
      workspace_id,
      tags,
      duration.num_seconds(),
      start,
      project_id,
      !non_billable,
    );

    self.post_time_entry(workspace_id, body)
  }

  fn post_time_entry(
    &self,
    workspace_id: WorkspaceId,
    body: serde_json::Value,
  ) -> Result<TimeEntry> {
    let uri = format!("workspaces/{workspace_id}/time_entries");
    self.request_with_body(Method::Post, &uri, body)
  }

  pub fn create_client(
    &self,
    name: &str,
    workspace_id: WorkspaceId,
  ) -> Result<Client> {
    let body = json!({
      "name": name,
      "workspace_id": workspace_id,
    });

    let uri = format!("workspaces/{workspace_id}/clients");

    self.request_with_body(Method::Post, &uri, body)
  }

  pub fn create_project(
    &self,
    name: &str,
    workspace_id: WorkspaceId,
    client_id: Option<crate::types::ClientId>,
    billable: bool,
    color: Option<&str>,
  ) -> Result<Project> {
    let mut body_map = serde_json::Map::new();
    body_map.insert("name".to_owned(), json!(name));
    body_map.insert("workspace_id".to_owned(), json!(workspace_id));
    body_map.insert("active".to_owned(), json!(true));
    body_map.insert("billable".to_owned(), json!(billable));

    if let Some(cid) = client_id {
      body_map.insert("client_id".to_owned(), json!(cid));
    }

    if let Some(color_value) = color {
      body_map.insert("color".to_owned(), json!(color_value));
    }

    let body = serde_json::Value::Object(body_map);

    let uri = format!("workspaces/{workspace_id}/projects");

    self.request_with_body(Method::Post, &uri, body)
  }

  #[allow(
    clippy::too_many_arguments,
    reason = "All parameters are necessary for the Toggl API"
  )]
  pub fn start_time_entry(
    &self,
    start: DateTime<Local>,
    workspace_id: WorkspaceId,
    description: Option<&str>,
    tags: Option<&[String]>,
    project_id: ProjectId,
    non_billable: bool,
  ) -> Result<TimeEntry> {
    // v9 convention: running entries use duration = -1
    let body = build_time_entry_body(
      description,
      workspace_id,
      tags,
      -1,
      start,
      project_id,
      !non_billable,
    );

    self.post_time_entry(workspace_id, body)
  }

  pub fn stop_time_entry(
    &self,
    workspace_id: WorkspaceId,
    time_entry_id: TimeEntryId,
  ) -> Result<TimeEntry> {
    self.request(
      Method::Patch,
      &format!("workspaces/{workspace_id}/time_entries/{time_entry_id}/stop"),
    )
  }

  pub fn delete_time_entry(
    &self,
    workspace_id: WorkspaceId,
    time_entry_id: TimeEntryId,
  ) -> Result<()> {
    self.empty_request(
      Method::Delete,
      &format!("workspaces/{workspace_id}/time_entries/{time_entry_id}"),
    )
  }

  pub fn get_time_entry(
    &self,
    time_entry_id: TimeEntryId,
  ) -> Result<TimeEntry> {
    self.request(Method::Get, &format!("me/time_entries/{time_entry_id}"))
  }

  pub fn get_current_time_entry(&self) -> Result<Option<TimeEntry>> {
    self.request(Method::Get, "me/time_entries/current")
  }

  pub fn update_time_entry(
    &self,
    time_entry_id: TimeEntryId,
    time_entry: &TimeEntry,
  ) -> Result<TimeEntry> {
    let workspace_id = time_entry.workspace_id;

    let mut body = serde_json::Map::new();
    body.insert("billable".to_owned(), json!(time_entry.billable));
    body.insert("description".to_owned(), json!(time_entry.description));
    body.insert("duration".to_owned(), json!(time_entry.duration));
    body.insert("project_id".to_owned(), json!(time_entry.project_id));
    body.insert("start".to_owned(), json!(time_entry.start));
    body.insert("tags".to_owned(), json!(time_entry.tags));
    body.insert("workspace_id".to_owned(), json!(workspace_id));

    // Toggl v9 rejects stop:null + negative duration as inconsistent;
    // omit stop entirely when the entry is still running.
    if let Some(stop) = time_entry.stop {
      body.insert("stop".to_owned(), json!(stop));
    }

    self.request_with_body(
      Method::Put,
      &format!("workspaces/{workspace_id}/time_entries/{time_entry_id}"),
      serde_json::Value::Object(body),
    )
  }
}

fn build_time_entry_body(
  description: Option<&str>,
  workspace_id: WorkspaceId,
  tags: Option<&[String]>,
  duration_seconds: i64,
  start: DateTime<Local>,
  project_id: ProjectId,
  billable: bool,
) -> serde_json::Value {
  // Normalize to UTC so the serialized form is stable regardless of the
  // machine's local timezone — keeps tests and Toggl's storage canonical.
  json!({
    "billable": billable,
    "created_with": CREATED_WITH,
    "description": description,
    "duration": duration_seconds,
    "project_id": project_id,
    "start": start.with_timezone(&Utc),
    "tags": tags,
    "workspace_id": workspace_id,
  })
}
