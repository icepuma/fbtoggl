use crate::config::read_settings;
use crate::model::Range;
use crate::model::ReportDetails;
use anyhow::anyhow;
use reqwest::blocking;
use reqwest::Method;
use reqwest::StatusCode;
use reqwest::Url;
use serde::de::DeserializeOwned;

pub struct TogglReportClient {
  base_url: Url,
  client: blocking::Client,
  api_token: String,
}

pub const CREATED_WITH: &str = "fbtoggl";

pub fn init_report_client() -> anyhow::Result<TogglReportClient> {
  let settings = read_settings()?;

  TogglReportClient::new(settings.api_token)
}

impl TogglReportClient {
  pub fn new(api_token: String) -> anyhow::Result<TogglReportClient> {
    #[cfg(not(test))]
    let base_url = "https://api.track.toggl.com/reports/api/v2/".parse()?;

    #[cfg(test)]
    let base_url = mockito::server_url().parse()?;

    let client = blocking::Client::new();

    Ok(TogglReportClient {
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

  fn empty_request_with_body<D: DeserializeOwned>(
    &self,
    method: Method,
    uri: &str,
  ) -> anyhow::Result<D> {
    let response = self.base_request(method, uri)?.send()?;

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

  pub fn details(
    &self,
    workspace_id: u64,
    range: &Range,
    page: u64,
  ) -> anyhow::Result<ReportDetails> {
    let (start, end) = range.as_range();

    let uri = format!(
      "details?workspace_id={}&user_agent={}&page={}&since={}&until={}",
      &workspace_id,
      urlencoding::encode(CREATED_WITH),
      &page,
      start.date().format("%Y-%m-%d"),
      end.date().format("%Y-%m-%d"),
    );

    self.empty_request_with_body(Method::GET, &uri)
  }
}
