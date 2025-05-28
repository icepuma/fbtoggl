use std::fmt::Debug;

use crate::config::read_settings;
use crate::model::Range;
use crate::model::ReportDetails;
use anyhow::anyhow;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use colored::Colorize;
use minreq::Method;
use minreq::Request;
use minreq::Response;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::json;
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
    let base_url = "https://api.track.toggl.com/reports/api/v3/".parse()?;

    Ok(TogglReportClient {
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

  fn request_with_body<D: DeserializeOwned + Debug, S: Serialize + Debug>(
    &self,
    debug: bool,
    method: Method,
    uri: &str,
    body: S,
  ) -> anyhow::Result<(Option<u64>, D)> {
    let request = self.base_request(method, uri)?.with_json(&body)?;

    if debug {
      println!("{}", "Request:".bold().underline());
      println!("{request:?}");
      println!();
      println!("{:?}", &body);
      println!();
    }

    let response = request.send()?;

    let next_id = response
      .headers
      .get("x-next-row-number")
      .map(|value| value.to_string())
      .and_then(|value| value.parse::<u64>().ok());

    self.response(debug, response).map(|body| (next_id, body))
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
    first_row_number: Option<u64>,
  ) -> anyhow::Result<(Option<u64>, Vec<ReportDetails>)> {
    let (start, end) = range.as_range()?;

    let uri = format!("workspace/{workspace_id}/search/time_entries");

    let body = json!({
      "start_date": start.naive_local().format("%Y-%m-%d").to_string(),
      "created_with": CREATED_WITH,
      "end_date":end.naive_local().format("%Y-%m-%d").to_string(),
      "first_row_number": first_row_number,
    });

    self.request_with_body(debug, Method::Post, &uri, body)
  }
}
