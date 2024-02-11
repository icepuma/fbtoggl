use crate::model::Range;
use chrono::{DateTime, Duration, Local};
use clap::{Parser, Subcommand, ValueEnum};
use jackdauer::duration;
use serde::Serialize;

pub const APP_NAME: &str = "fbtoggl";

#[derive(Parser)]
#[command(author, about, version)]
pub struct Options {
  #[arg(long, value_enum, default_value_t = Format::Raw)]
  pub format: Format,

  /// Show debug information -> log HTTP requests and responses
  #[arg(long)]
  pub debug: bool,

  #[clap(subcommand)]
  pub subcommand: SubCommand,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Format {
  Json,
  Raw,
  Table,
}

#[derive(Subcommand, Debug)]
pub enum SubCommand {
  /// (deprecated: use 'fbtoggl settings init') Initialize settings
  Init,

  #[command(subcommand, about = "Settings")]
  Settings(Settings),

  #[command(subcommand, about = "Workspaces")]
  Workspaces(Workspaces),

  #[command(subcommand, about = "Projects (default workspace)")]
  Projects(Projects),

  #[command(subcommand, about = "Time entries")]
  TimeEntries(TimeEntries),

  #[command(subcommand, about = "Clients (default workspace)")]
  Clients(Clients),

  #[command(subcommand, about = "Reports")]
  Reports(Reports),
}

#[derive(Subcommand, Debug)]
pub enum Settings {
  /// Initialize settings
  Init,
}

#[derive(Subcommand, Debug)]
pub enum Reports {
  /// Detailed report with violations: more than 10 hours, start before 6am, end after 10pm and pause violations (Arbeitszeitgesetz (ArbZG) § 4 Ruhepausen)
  Detailed(Detailed),
}

#[derive(Parser, Debug)]
pub struct Detailed {
  /// Start ('today', 'yesterday', 'this-week', 'last-week', 'this-month', 'last-month', ISO 8601 date '2021-11-01'), ISO 8601 date range '2021-11-01|2021-11-02')
  #[arg(long, default_value = "today")]
  pub range: Range,
}

#[derive(Subcommand, Debug)]
pub enum Workspaces {
  /// List all workspaces
  List,
}

#[derive(Parser, Debug)]
pub struct ListProjects {
  /// Include archived projects
  #[arg(long, default_value_t = false)]
  pub include_archived: bool,
}

#[derive(Subcommand, Debug)]
pub enum Projects {
  /// List all projects (default workspace)
  List(ListProjects),
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
  #[arg(long, default_value = "today")]
  pub range: Range,

  /// Show days which have no entry (monday, tuesday, wednesday, thursday and friday only)
  #[arg(long)]
  pub missing: bool,
}

#[derive(Parser, Debug)]
pub struct CreateClient {
  /// Name of the client
  #[arg(long)]
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
  #[arg(long)]
  pub project: String,

  /// Description of the timer
  #[arg(long)]
  pub description: Option<String>,

  /// Tags
  #[arg(long)]
  pub tags: Option<Vec<String>>,

  /// Duration ('1 hour', '10 minutes', '1 hour 12 minutes')
  #[arg(
    long,
    value_parser = parse_duration,
    conflicts_with = "end"
  )]
  pub duration: Option<Duration>,

  /// Lunch break (if set, adds a lunch break of 1 hour)
  #[arg(long)]
  pub lunch_break: bool,

  /// Start (e.g. 'now', 'today at 6am', 'yesterday at 16:30' '2021-11-30T06:00', '2 hours ago', 'yesterday at 6am') - All possible formats https://github.com/PicoJr/htp/blob/HEAD/src/time.pest
  #[arg(
    long,
    default_value = "now",
    value_parser = parse_time,
  )]
  pub start: DateTime<Local>,

  /// End (e.g. 'now', 'today at 6am', 'yesterday at 16:30' '2021-11-30T06:00', '2 hours ago', 'yesterday at 6am') - All possible formats https://github.com/PicoJr/htp/blob/HEAD/src/time.pest
  #[arg(
    long,
    value_parser = parse_time,
    conflicts_with = "duration"
  )]
  pub end: Option<DateTime<Local>>,

  /// Time entry is non-billable
  #[arg(long)]
  pub non_billable: bool,
}

#[derive(Parser, Debug)]
pub struct StartTimeEntry {
  /// Name of the project
  #[arg(long)]
  pub project: String,

  /// Description of the timer
  #[arg(long)]
  pub description: Option<String>,

  /// Tags
  #[arg(long)]
  pub tags: Option<Vec<String>>,

  /// Time entry is non-billable
  #[arg(long)]
  pub non_billable: bool,
}

#[derive(Parser, Debug)]
pub struct StopTimeEntry {
  /// Id of the time entry
  #[arg(long)]
  pub id: u64,
}

#[derive(Parser, Debug)]
pub struct DeleteTimeEntry {
  /// Id of the time entry
  #[arg(long)]
  pub id: u64,
}

#[derive(Parser, Debug)]
pub struct TimeEntryDetails {
  /// Id of the time entry
  #[arg(long)]
  pub id: u64,
}

#[derive(Parser, Debug)]
pub struct ListClients {
  /// Include archived
  #[arg(long, default_value_t = false)]
  pub include_archived: bool,
}

#[derive(Parser, Debug)]
pub enum Clients {
  /// List all clients (default workspace)
  List(ListClients),

  /// Create client (in default workspace)
  Create(CreateClient),
}

pub(crate) fn output_values_json<T: Serialize>(values: &[T]) {
  for value in values {
    if let Ok(output) = serde_json::to_string_pretty(&value) {
      println!("{output}");
    }
  }
}
