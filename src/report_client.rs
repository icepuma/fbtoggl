use std::fmt::Debug;

use crate::config::read_settings;
use crate::model::Range;
use crate::model::ReportDetails;
use anyhow::anyhow;
use colored::Colorize;
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

  fn empty_request_with_body<D: DeserializeOwned + Debug>(
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

  fn response<D: DeserializeOwned + Debug>(
    &self,
    debug: bool,
    response: blocking::Response,
  ) -> anyhow::Result<D> {
    if debug {
      println!("{}", "Response:".bold().underline());
      println!("{response:?}");
      println!();
    }

    match response.status() {
      StatusCode::OK | StatusCode::CREATED if debug => match response.json() {
        Ok(json) => {
          println!("{}", "Received JSON response:".bold().underline());
          println!("{json:?}");
          println!();

          Ok(json)
        }
        Err(err) => Err(anyhow!("Failed to deserialize JSON: {}", err)),
      },
      StatusCode::OK | StatusCode::CREATED => Ok(response.json()?),
      status => match response.text() {
        Ok(text) => Err(anyhow!("{} - {}", status, text)),
        Err(_) => Err(anyhow!("{}", status)),
      },
    }
  }

  pub fn details(
    &self,
    debug: bool,
    workspace_id: u64,
    range: &Range,
    page: u64,
  ) -> anyhow::Result<ReportDetails> {
    let (start, end) = range.as_range()?;

    let uri = format!(
      "details?workspace_id={}&user_agent={}&page={}&since={}&until={}",
      &workspace_id,
      urlencoding::encode(CREATED_WITH),
      &page,
      start.naive_local().format("%Y-%m-%d"),
      end.naive_local().format("%Y-%m-%d"),
    );

    self.empty_request_with_body(debug, Method::GET, &uri)
  }
}
