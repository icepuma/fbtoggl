use crate::model::Range;
use chrono::{DateTime, Duration, Local};
use clap::{ArgEnum, Parser, Subcommand};
use jackdauer::duration;
use serde::Serialize;

pub const APP_NAME: &str = "fbtoggl";

#[derive(Parser)]
#[clap(author, about, version)]
pub struct Options {
  #[clap(long, arg_enum, value_parser, default_value = "raw")]
  pub format: Format,

  #[clap(subcommand)]
  pub subcommand: SubCommand,
}

#[derive(Debug, Clone, ArgEnum)]
pub enum Format {
  Json,
  Raw,
  Table,
}

#[derive(Subcommand, Debug)]
pub enum SubCommand {
  /// Initialize settings
  Init,

  /// Workspaces
  #[clap(subcommand)]
  Workspaces(Workspaces),

  /// Projects (default workspace)
  #[clap(subcommand)]
  Projects(Projects),

  /// Time entries
  #[clap(subcommand)]
  TimeEntries(TimeEntries),

  /// Clients (default workspace)
  #[clap(subcommand)]
  Clients(Clients),

  /// Reports
  #[clap(subcommand)]
  Reports(Reports),
}

#[derive(Subcommand, Debug)]
pub enum Reports {
  /// Detailed report with violations: more than 10 hours, start before 6am, end after 10pm and pause violations (Arbeitszeitgesetz (ArbZG) § 4 Ruhepausen)
  Detailed(Detailed),
}

#[derive(Parser, Debug)]
pub struct Detailed {
  /// Start ('today', 'yesterday', 'this-week', 'last-week', 'this-month', 'last-month', ISO 8601 date '2021-11-01'), ISO 8601 date range '2021-11-01|2021-11-02')
  #[clap(long, default_value = "today", value_parser)]
  pub range: Range,
}

#[derive(Subcommand, Debug)]
pub enum Workspaces {
  /// List all workspaces
  List,
}

#[derive(Subcommand, Debug)]
pub enum Projects {
  /// List all projects (default workspace)
  List,
}

#[derive(Parser, Debug)]
pub enum TimeEntries {
  /// List all time entries
  List(ListTimeEntries),

  /// Create time entry (billable by default). The combination of --end and --duration conflicts!
  Create(CreateTimeEntry),

  /// Start a time entry (billable by default)
  Start(StartTimeEntry),

  /// Stop a time entry
  Stop(StopTimeEntry),

  /// Delete time entry
  Delete(DeleteTimeEntry),
}

#[derive(Parser, Debug)]
pub struct ListTimeEntries {
  /// Start ('today', 'yesterday', 'this-week', 'last-week', 'this-month', 'last-month', ISO 8601 date '2021-11-01'), ISO 8601 date range '2021-11-01|2021-11-02')
  #[clap(long, default_value = "today", value_parser)]
  pub range: Range,

  /// Show days which have no entry (monday, tuesday, wednesday, thursday and friday only)
  #[clap(long, value_parser)]
  pub missing: bool,
}

#[derive(Parser, Debug)]
pub struct CreateClient {
  /// Name of the client
  #[clap(long, value_parser)]
  pub name: String,
}

fn parse_duration(duration_to_parse: &str) -> anyhow::Result<Duration> {
  let duration = duration(duration_to_parse)?;
  Ok(Duration::from_std(duration)?)
}

fn parse_time(time_to_parse: &str) -> anyhow::Result<DateTime<Local>> {
  let now = Local::now();
  Ok(htp::parse(time_to_parse, now)?)
}

#[derive(Parser, Debug)]
pub struct CreateTimeEntry {
  /// Name of the project
  #[clap(long, value_parser)]
  pub project: String,

  /// Description of the timer
  #[clap(long, value_parser)]
  pub description: Option<String>,

  /// Tags
  #[clap(long, value_parser)]
  pub tags: Option<Vec<String>>,

  /// Duration ('1 hour', '10 minutes', '1 hour 12 minutes')
  #[clap(
    long,
    value_parser = parse_duration,
    conflicts_with = "end"
  )]
  pub duration: Option<Duration>,

  /// Lunch break (if set, adds a lunch break of 1 hour)
  #[clap(long, value_parser)]
  pub lunch_break: bool,

  /// Start (e.g. 'now', 'today at 6am', 'yesterday at 16:30' '2021-11-30T06:00', '2 hours ago', 'yesterday at 6am') - All possible formats https://github.com/PicoJr/htp/blob/HEAD/src/time.pest
  #[clap(
    long,
    default_value = "now",
    value_parser = parse_time,
  )]
  pub start: DateTime<Local>,

  /// End (e.g. 'now', 'today at 6am', 'yesterday at 16:30' '2021-11-30T06:00', '2 hours ago', 'yesterday at 6am') - All possible formats https://github.com/PicoJr/htp/blob/HEAD/src/time.pest
  #[clap(
    long,
    value_parser = parse_time,
    conflicts_with = "duration"
  )]
  pub end: Option<DateTime<Local>>,

  /// Time entry is non-billable
  #[clap(long, value_parser)]
  pub non_billable: bool,
}

#[derive(Parser, Debug)]
pub struct StartTimeEntry {
  /// Name of the project
  #[clap(long, value_parser)]
  pub project: String,

  /// Description of the timer
  #[clap(long, value_parser)]
  pub description: Option<String>,

  /// Tags
  #[clap(long, value_parser)]
  pub tags: Option<Vec<String>>,

  /// Time entry is non-billable
  #[clap(long, value_parser)]
  pub non_billable: bool,
}

#[derive(Parser, Debug)]
pub struct StopTimeEntry {
  /// Id of the time entry
  #[clap(long, value_parser)]
  pub id: u64,
}

#[derive(Parser, Debug)]
pub struct DeleteTimeEntry {
  /// Id of the time entry
  #[clap(long, value_parser)]
  pub id: u64,
}

#[derive(Parser, Debug)]
pub struct TimeEntryDetails {
  /// Id of the time entry
  #[clap(long, value_parser)]
  pub id: u64,
}

#[derive(Parser, Debug)]
pub enum Clients {
  /// List all clients (default workspace)
  List,

  /// Create client (in default workspace)
  Create(CreateClient),
}

pub(crate) fn output_values_json<T: Serialize>(values: &[T]) {
  for value in values {
    if let Ok(output) = serde_json::to_string_pretty(&value) {
      println!("{}", output);
    }
  }
}
