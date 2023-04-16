use std::fmt::Debug;

use crate::config::read_settings;
use crate::model::Range;
use crate::model::ReportDetails;
use anyhow::anyhow;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use colored::Colorize;
use minreq::Method;
use minreq::Request;
use minreq::Response;
use serde::de::DeserializeOwned;
use url::Url;

pub struct TogglReportClient {
  base_url: Url,
  api_token: String,
}

pub const CREATED_WITH: &str = "fbtoggl (https://github.com/icepuma/fbtoggl)";

const AUTHORIZATION: &str = "Authorization";

pub fn init_report_client() -> anyhow::Result<TogglReportClient> {
  let settings = read_settings()?;

  TogglReportClient::new(settings.api_token)
}

impl TogglReportClient {
  pub fn new(api_token: String) -> anyhow::Result<TogglReportClient> {
    let base_url = "https://api.track.toggl.com/reports/api/v2/".parse()?;

    Ok(TogglReportClient {
      base_url,
      api_token,
    })
  }

  fn basic_auth(&self) -> (String, String) {
    (
      AUTHORIZATION.to_string(),
      STANDARD.encode(format!("{}:api_token", &self.api_token)),
    )
  }

  fn base_request(&self, method: Method, uri: &str) -> anyhow::Result<Request> {
    let url = self.base_url.join(uri)?;

    let (key, value) = self.basic_auth();

    Ok(minreq::Request::new(method, url).with_header(key, value))
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

    self.empty_request_with_body(debug, Method::Get, &uri)
  }
}
