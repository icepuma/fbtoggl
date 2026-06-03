use crate::config::read_settings;
use crate::error::{Result, TogglError};
use crate::http_client::{HttpClient, check_status};
use crate::model::Range;
use crate::model::ReportDetails;
use crate::types::{ApiToken, WorkspaceId};
use anyhow::Context;
use colored::Colorize;
use minreq::Method;
use serde_json::json;
use url::Url;

pub struct TogglReportClient {
  base_url: Url,
  api_token: ApiToken,
  debug: bool,
}

pub fn init_report_client(debug: bool) -> anyhow::Result<TogglReportClient> {
  let settings =
    read_settings().context("Failed to read configuration settings")?;

  TogglReportClient::new(settings.api_token, debug)
    .context("Failed to initialize Toggl Reports client")
}

impl TogglReportClient {
  pub fn new(api_token: ApiToken, debug: bool) -> anyhow::Result<Self> {
    let base_url = "https://api.track.toggl.com/reports/api/v3/".parse()?;

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

  fn debug(&self) -> bool {
    self.debug
  }
}

impl TogglReportClient {
  pub fn detailed(
    &self,
    workspace_id: WorkspaceId,
    range: &Range,
  ) -> Result<Vec<ReportDetails>> {
    let mut result: Vec<ReportDetails> = vec![];
    let (start_date, end_date) = range.as_dates()?;

    let start_date = start_date.format("%Y-%m-%d").to_string();
    let end_date = end_date.format("%Y-%m-%d").to_string();

    let mut next_row: Option<u64> = None;
    let mut next_id: Option<i64> = None;

    loop {
      let mut body = serde_json::Map::new();
      body.insert("start_date".to_owned(), json!(start_date));
      body.insert("end_date".to_owned(), json!(end_date));
      if let Some(row) = next_row {
        body.insert("first_row_number".to_owned(), json!(row));
      }
      if let Some(id) = next_id {
        body.insert("first_id".to_owned(), json!(id));
      }

      let request = self
        .base_request(
          Method::Post,
          &format!("workspace/{workspace_id}/search/time_entries"),
        )?
        .with_json(&serde_json::Value::Object(body))?;

      let response = request.send()?;

      if self.debug {
        println!("{}", "Response:".bold().underline());
        println!("{response:?}");
        println!();
      }

      let row_header = response.header("x-next-row-number").map(str::to_owned);
      let id_header = response.header("x-next-id").map(str::to_owned);

      let response = check_status(response, self.service_name())?;
      let data: Vec<ReportDetails> = response
        .json()
        .map_err(|e| TogglError::Other(anyhow::anyhow!("JSON error: {e}")))?;
      result.extend(data);

      next_row = row_header.and_then(|v| v.parse::<u64>().ok());
      next_id = id_header.and_then(|v| v.parse::<i64>().ok());

      if next_row.is_none() && next_id.is_none() {
        break;
      }
    }

    Ok(result)
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used, reason = "Test code can panic on failure")]
  #![allow(
    clippy::significant_drop_tightening,
    reason = "Test server must remain alive for the duration of tests"
  )]

  use super::TogglReportClient;
  use crate::model::Range;
  use crate::types::WorkspaceId;
  use chrono::NaiveDate;
  use mockito::Matcher;
  use serde_json::json;

  #[test]
  fn detailed_uses_inclusive_report_end_date_for_single_day_range()
  -> anyhow::Result<()> {
    let mut server = mockito::Server::new();
    let request_body = json!({
      "start_date": "2021-11-21",
      "end_date": "2021-11-21",
    });

    let mock = server
      .mock("POST", "/workspace/42/search/time_entries")
      .with_header(
        "Authorization",
        "Basic Y2I3YmY3ZWZhNmQ2NTIwNDZhYmQyZjdkODRlZTE4YzE6YXBpX3Rva2Vu",
      )
      .with_status(200)
      .match_body(Matcher::Json(request_body))
      .with_body(json!([]).to_string())
      .expect(1)
      .create();

    {
      let client = TogglReportClient::new_with_base_url(
        "cb7bf7efa6d652046abd2f7d84ee18c1".to_owned(),
        server.url().parse()?,
      )?;

      client.detailed(
        WorkspaceId::new(42),
        &Range::Date(NaiveDate::from_ymd_opt(2021, 11, 21).unwrap()),
      )?;
    }

    mock.assert();

    Ok(())
  }

  #[test]
  fn detailed_uses_inclusive_report_end_date_for_from_to_range()
  -> anyhow::Result<()> {
    let mut server = mockito::Server::new();
    let request_body = json!({
      "start_date": "2021-11-01",
      "end_date": "2021-11-03",
    });

    let mock = server
      .mock("POST", "/workspace/42/search/time_entries")
      .with_header(
        "Authorization",
        "Basic Y2I3YmY3ZWZhNmQ2NTIwNDZhYmQyZjdkODRlZTE4YzE6YXBpX3Rva2Vu",
      )
      .with_status(200)
      .match_body(Matcher::Json(request_body))
      .with_body(json!([]).to_string())
      .expect(1)
      .create();

    {
      let client = TogglReportClient::new_with_base_url(
        "cb7bf7efa6d652046abd2f7d84ee18c1".to_owned(),
        server.url().parse()?,
      )?;

      client.detailed(
        WorkspaceId::new(42),
        &Range::FromTo(
          NaiveDate::from_ymd_opt(2021, 11, 1).unwrap(),
          NaiveDate::from_ymd_opt(2021, 11, 3).unwrap(),
        ),
      )?;
    }

    mock.assert();

    Ok(())
  }
}
