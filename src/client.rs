pub use crate::common::CREATED_WITH;
use crate::config::read_settings;
use crate::error::Result;
use crate::http_client::{HttpClient, HttpClientExt};
use crate::model::Client;
use crate::model::Me;
use crate::model::Project;
use crate::model::Range;
use crate::model::Workspace;
use crate::model::{TimeEntry, TimeEntryDetail};
use crate::types::{
  ApiToken, ClientStatusFilter, ProjectId, TimeEntryId, WorkspaceId,
};
use anyhow::Context;
use chrono::DateTime;
use chrono::Duration;
use chrono::Local;
use minreq::Method;
use serde_json::json;
use url::Url;

pub struct TogglClient {
  base_url: Url,
  api_token: ApiToken,
}

pub fn init_client() -> anyhow::Result<TogglClient> {
  let settings =
    read_settings().context("Failed to read configuration settings")?;

  let api_token =
    ApiToken::new(settings.api_token).context("Invalid API token")?;

  TogglClient::new(api_token).context("Failed to initialize Toggl client")
}

impl TogglClient {
  pub fn new(api_token: ApiToken) -> anyhow::Result<Self> {
    let base_url = "https://api.track.toggl.com/api/v9/"
      .parse()
      .context("Failed to parse Toggl API base URL")?;

    Ok(Self {
      base_url,
      api_token,
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
}

impl TogglClient {
  pub fn get_workspace_clients(
    &self,
    debug: bool,
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

    self.request(debug, Method::Get, &uri)
  }

  #[allow(
    clippy::arithmetic_side_effects,
    reason = "Date arithmetic is necessary for API date range calculation"
  )]
  pub fn get_time_entries(
    &self,
    debug: bool,
    range: &Range,
  ) -> Result<Vec<TimeEntry>> {
    let (start, end) = range.as_range()?;
    let start_date = start.format("%Y-%m-%d").to_string();

    // End date is not inclusive, therefore we add one day
    let end_date = (end
      + Duration::try_days(1)
        .ok_or_else(|| anyhow::anyhow!("Failed to add one day to end date"))?)
    .format("%Y-%m-%d")
    .to_string();

    let uri = format!(
      "me/time_entries?start_date={}&end_date={}",
      urlencoding::encode(&start_date),
      urlencoding::encode(&end_date),
    );

    self.request::<Vec<TimeEntry>>(debug, Method::Get, &uri)
  }

  pub fn get_workspaces(&self, debug: bool) -> Result<Vec<Workspace>> {
    self.request::<Vec<Workspace>>(debug, Method::Get, "workspaces")
  }

  pub fn get_me(&self, debug: bool) -> Result<Me> {
    self.request::<Me>(debug, Method::Get, "me")
  }

  pub fn get_workspace_projects(
    &self,
    debug: bool,
    include_archived: bool,
    workspace_id: WorkspaceId,
  ) -> Result<Vec<Project>> {
    let uri = if include_archived {
      format!("workspaces/{workspace_id}/projects")
    } else {
      format!("workspaces/{workspace_id}/projects?active=true")
    };

    self.request::<Vec<Project>>(debug, Method::Get, &uri)
  }

  #[allow(
    clippy::too_many_arguments,
    reason = "All parameters are necessary for the Toggl API"
  )]
  pub fn create_time_entry(
    &self,
    debug: bool,
    description: Option<&str>,
    workspace_id: WorkspaceId,
    tags: Option<&[String]>,
    duration: Duration,
    start: DateTime<Local>,
    project_id: ProjectId,
    non_billable: bool,
  ) -> Result<TimeEntry> {
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
    workspace_id: WorkspaceId,
  ) -> Result<Client> {
    let body = json!({
      "active": true,
      "name": name,
      "wid": workspace_id,
    });

    let uri = format!("workspaces/{workspace_id}/clients");

    self.request_with_body(debug, Method::Post, &uri, body)
  }

  pub fn create_project(
    &self,
    debug: bool,
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

    self.request_with_body(debug, Method::Post, &uri, body)
  }

  #[allow(
    clippy::too_many_arguments,
    reason = "All parameters are necessary for the Toggl API"
  )]
  pub fn start_time_entry(
    &self,
    debug: bool,
    start: DateTime<Local>,
    workspace_id: WorkspaceId,
    description: Option<&str>,
    tags: Option<&[String]>,
    project_id: ProjectId,
    non_billable: bool,
  ) -> Result<TimeEntry> {
    let billable = !non_billable;
    let duration = start.timestamp().saturating_neg();

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

    self.request_with_body(debug, Method::Post, "time_entries", body)
  }

  pub fn stop_time_entry(
    &self,
    debug: bool,
    workspace_id: WorkspaceId,
    time_entry_id: TimeEntryId,
  ) -> Result<TimeEntry> {
    self.request(
      debug,
      Method::Patch,
      &format!("workspaces/{workspace_id}/time_entries/{time_entry_id}/stop"),
    )
  }

  pub fn delete_time_entry(
    &self,
    debug: bool,
    time_entry_id: TimeEntryId,
  ) -> Result<()> {
    self.empty_request(
      debug,
      Method::Delete,
      &format!("time_entries/{time_entry_id}"),
    )
  }

  pub fn get_time_entry(
    &self,
    debug: bool,
    time_entry_id: TimeEntryId,
  ) -> Result<TimeEntry> {
    let detail: TimeEntryDetail = self.request(
      debug,
      Method::Get,
      &format!("me/time_entries/{time_entry_id}"),
    )?;
    Ok(detail.into())
  }

  pub fn get_current_time_entry(
    &self,
    debug: bool,
  ) -> Result<Option<TimeEntry>> {
    let result: Option<TimeEntry> =
      self.request(debug, Method::Get, "me/time_entries/current")?;
    Ok(result)
  }

  pub fn update_time_entry(
    &self,
    debug: bool,
    time_entry_id: TimeEntryId,
    time_entry: &TimeEntry,
  ) -> Result<TimeEntry> {
    let me = self.get_me(debug)?;
    let workspace_id = me.default_workspace_id;

    let body = json!({
      "billable": time_entry.billable,
      "description": time_entry.description,
      "duration": time_entry.duration,
      "pid": time_entry.pid,
      "start": time_entry.start,
      "stop": time_entry.stop,
      "tags": time_entry.tags,
      "wid": workspace_id
    });

    self.request_with_body(
      debug,
      Method::Put,
      &format!("workspaces/{workspace_id}/time_entries/{time_entry_id}"),
      body,
    )
  }
}
