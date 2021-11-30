use crate::model::{Range, Start};
use chrono::Duration;
use clap::{ArgEnum, Parser};
use jackdauer::duration;
use serde::Serialize;

pub const APP_NAME: &str = "fbtoggl";

#[derive(Parser)]
#[clap(author, about, version)]
pub struct Options {
  #[clap(long, arg_enum, default_value = "raw")]
  pub format: Format,

  #[clap(subcommand)]
  pub subcommand: SubCommand,
}

#[derive(ArgEnum, Debug, Clone)]
pub enum Format {
  Json,
  Raw,
  Table,
}

#[derive(Parser)]
pub enum SubCommand {
  #[clap(about = "Init settings")]
  Init,

  #[clap(subcommand, about = "Workspaces")]
  Workspaces(Workspaces),

  #[clap(subcommand, about = "Projects (default workspace)")]
  Projects(Projects),

  #[clap(subcommand, about = "Time entries")]
  TimeEntries(TimeEntries),

  #[clap(subcommand, about = "Clients (default workspace)")]
  Clients(Clients),
}

#[derive(Parser, Debug)]
pub enum Workspaces {
  #[clap(about = "List all workspaces")]
  List,
}

#[derive(Parser, Debug)]
pub enum Projects {
  #[clap(about = "List all projects (default workspace)")]
  List,
}

#[derive(Parser, Debug)]
pub enum TimeEntries {
  #[clap(about = "List all time entries")]
  List(ListTimeEntries),

  #[clap(about = "Create time entry (billable by default)")]
  Create(CreateTimeEntry),

  #[clap(about = "Start a time entry (billable by default)")]
  Start(StartTimeEntry),

  #[clap(about = "Stop a time entry")]
  Stop(StopTimeEntry),
}

#[derive(Parser, Debug)]
pub struct ListTimeEntries {
  #[clap(
    long,
    about = "Start ('today', 'yesterday', 'this-week', 'last-week', 'this-month', 'last-month', ISO 8601 date '2021-11-01'), ISO 8601 date range '2021-11-01|2021-11-02'",
    default_value = "today"
  )]
  pub range: Range,
}

#[derive(Parser, Debug)]
pub struct CreateClient {
  #[clap(long, about = "Name of the client")]
  pub name: String,
}

fn parse_duration(duration_to_parse: &str) -> anyhow::Result<Duration> {
  let bla = duration(duration_to_parse)?;
  Ok(Duration::from_std(bla)?)
}

#[derive(Parser, Debug)]
pub struct CreateTimeEntry {
  #[clap(long, about = "Name of the project")]
  pub project: String,

  #[clap(long, about = "Description of the timer")]
  pub description: String,

  #[clap(long, about = "Tags")]
  pub tags: Option<Vec<String>>,

  #[clap(long, about = "Duration ('1 hour', '10 minutes', '1 hour 12 minutes')", parse(try_from_str = parse_duration))]
  pub duration: Duration,

  #[clap(long, about = "Lunch break (if set, adds a lunch break of 1 hour)")]
  pub lunch_break: bool,

  #[clap(
    long,
    about = "Start ('now', ISO 8601 date time '2021-11-01T00:00:00+01:00')",
    default_value = "now"
  )]
  pub start: Start,

  #[clap(long, about = "Time entry is non-billable")]
  pub non_billable: bool,
}

#[derive(Parser, Debug)]
pub struct StartTimeEntry {
  #[clap(long, about = "Name of the project")]
  pub project: String,

  #[clap(long, about = "Description of the timer")]
  pub description: String,

  #[clap(long, about = "Tags")]
  pub tags: Option<Vec<String>>,

  #[clap(long, about = "Time entry is non-billable")]
  pub non_billable: bool,
}

#[derive(Parser, Debug)]
pub struct StopTimeEntry {
  #[clap(long, about = "Id of the time entry")]
  pub id: u64,

  #[clap(long, about = "Name of the project")]
  pub project: String,

  #[clap(long, about = "Description of the timer")]
  pub description: String,

  #[clap(long, about = "Tags")]
  pub tags: Option<Vec<String>>,
}

#[derive(Parser, Debug)]
pub enum Clients {
  #[clap(about = "List all clients (default workspace)")]
  List,

  #[clap(about = "Create client (in default workspace)")]
  Create(CreateClient),
}

pub(crate) fn output_values_json<T: Serialize>(values: &[T]) {
  for value in values {
    if let Ok(output) = serde_json::to_string_pretty(&value) {
      println!("{}", output);
    }
  }
}
