use crate::common::CREATED_WITH;
use crate::config::read_settings;
use crate::error::Result;
use crate::http_client::{HttpClient, ResponseExt};
use crate::model::Range;
use crate::model::ReportDetails;
use crate::types::{ApiToken, WorkspaceId};
use anyhow::Context;
use minreq::Method;
use serde_json::json;
use url::Url;

pub struct TogglReportClient {
  base_url: Url,
  api_token: ApiToken,
}

pub fn init_report_client() -> anyhow::Result<TogglReportClient> {
  let settings =
    read_settings().context("Failed to read configuration settings")?;

  let api_token =
    ApiToken::new(settings.api_token).context("Invalid API token")?;

  TogglReportClient::new(api_token)
    .context("Failed to initialize Toggl Reports client")
}

impl TogglReportClient {
  pub fn new(api_token: ApiToken) -> anyhow::Result<Self> {
    let base_url = "https://api.track.toggl.com/reports/api/v3/".parse()?;

    Ok(Self {
      base_url,
      api_token,
    })
  }
}

impl HttpClient for TogglReportClient {
  fn base_url(&self) -> &Url {
    &self.base_url
  }

  fn api_token(&self) -> &ApiToken {
    &self.api_token
  }

  fn service_name(&self) -> &'static str {
    "Toggl Reports"
  }
}

impl TogglReportClient {
  #[allow(
    clippy::arithmetic_side_effects,
    reason = "Date arithmetic is necessary for API date range calculation"
  )]
  pub fn detailed(
    &self,
    workspace_id: WorkspaceId,
    range: &Range,
    debug: bool,
  ) -> Result<Vec<ReportDetails>> {
    let mut result: Vec<ReportDetails> = vec![];
    let (start_date, end_date) = range.as_range()?;

    let start_date = start_date.format("%Y-%m-%d").to_string();
    let end_date = end_date.format("%Y-%m-%d").to_string();

    let mut next_row: Option<u64> = None;

    loop {
      let body = json!({
        "start_date": start_date,
        "end_date": end_date,
        "user_agent": CREATED_WITH,
        "first_row_number": next_row,
      });

      let request = self
        .base_request(
          Method::Post,
          &format!("workspace/{workspace_id}/search/time_entries"),
        )?
        .with_json(&body)?;

      let response = request.send()?;

      let (data, next_id) = response.handle_with_header::<Vec<ReportDetails>>(
        debug,
        "x-next-row-number",
        self.service_name(),
      )?;

      result.extend(data);

      match next_id {
        Some(value) => next_row = value.parse::<u64>().ok(),
        None => break,
      }

      if next_row.is_none() {
        break;
      }
    }

    Ok(result)
  }
}
