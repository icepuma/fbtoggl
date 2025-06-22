use crate::model::Range;
use crate::types::TimeEntryId;
use chrono::{DateTime, Duration, Local};
use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::{Generator, Shell, generate};
use jackdauer::duration;
use serde::Serialize;
use std::io;

pub const APP_NAME: &str = "fbtoggl";

#[derive(Parser)]
#[command(author, about, version)]
pub struct Options {
  #[arg(long, value_enum, default_value_t = Format::Raw)]
  pub format: Format,

  /// Show debug information -> log HTTP requests and responses
  /// WARNING: This will display your API token in the Authorization header!
  #[arg(long)]
  pub debug: bool,

  #[clap(subcommand)]
  pub subcommand: Option<SubCommand>,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Format {
  Json,
  Raw,
  Table,
}

#[derive(Subcommand, Debug)]
pub enum SubCommand {
  /// Start a time entry (billable by default)
  Start(StartTimeEntry),

  /// Stop current time entry
  Stop(StopTimeEntry),

  /// Continue last time entry or specific entry by ID
  Continue(ContinueTimeEntry),

  /// Show current running time entry
  Current,

  /// Add completed time entry (billable by default)
  Add(CreateTimeEntry),

  /// Show time entries (defaults to today)
  Log(ListTimeEntries),

  /// Show time entry details
  Show(TimeEntryDetails),

  /// Edit time entry
  Edit(EditTimeEntry),

  /// Delete time entry
  Delete {
    /// Time entry ID
    id: TimeEntryId,
  },

  /// Detailed report with violations
  Report(ReportOptions),

  /// Summary statistics
  Summary(SummaryOptions),

  #[command(subcommand, about = "Workspace management")]
  Workspace(Workspace),

  #[command(subcommand, about = "Project management")]
  Project(Project),

  #[command(subcommand, about = "Client management")]
  Client(Client),

  #[command(subcommand, about = "Configuration")]
  Config(Config),

  /// (deprecated: use 'fbtoggl config init') Initialize settings
  Init,

  /// Generate shell completions
  Completions {
    /// Shell type
    shell: Shell,
  },
}

#[derive(Subcommand, Debug)]
pub enum Config {
  /// Initialize settings
  Init,

  /// Show current configuration
  Show,

  /// Set configuration value
  Set {
    /// Configuration key
    key: String,
    /// Configuration value
    value: String,
  },
}

#[derive(Parser, Debug, Clone, Copy)]
pub struct ReportOptions {
  /// Date range ('today', 'yesterday', 'this-week', 'last-week', 'this-month', 'last-month', ISO 8601 date '2021-11-01', ISO 8601 date range '2021-11-01|2021-11-02')
  #[arg(long, default_value = "today")]
  pub range: Range,
}

#[derive(Parser, Debug, Clone, Copy)]
pub struct SummaryOptions {
  /// Date range ('today', 'yesterday', 'this-week', 'last-week', 'this-month', 'last-month', ISO 8601 date '2021-11-01', ISO 8601 date range '2021-11-01|2021-11-02')
  #[arg(long, default_value = "this-week")]
  pub range: Range,
}

#[derive(Subcommand, Debug, Clone, Copy)]
pub enum Workspace {
  /// List all workspaces
  List,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Project {
  /// List all projects
  List {
    /// Include all (including archived)
    #[arg(long)]
    all: bool,
  },

  /// Create new project
  Create {
    /// Project name
    #[arg(long)]
    name: String,

    /// Client name (optional)
    #[arg(long)]
    client: Option<String>,

    /// Project is billable by default
    #[arg(long)]
    billable: bool,

    /// Project color (optional)
    #[arg(long)]
    color: Option<String>,
  },
}

#[derive(Subcommand, Debug)]
pub enum Client {
  /// List all clients
  List {
    /// Include all (including archived)
    #[arg(long)]
    all: bool,
  },

  /// Create new client
  Create {
    /// Client name
    #[arg(long)]
    name: String,
  },
}

#[derive(Parser, Debug, Clone)]
pub struct ListTimeEntries {
  /// Date range ('today', 'yesterday', 'this-week', 'last-week', 'this-month', 'last-month', ISO 8601 date '2021-11-01', ISO 8601 date range '2021-11-01|2021-11-02')
  #[arg(long, default_value = "today")]
  pub range: Range,

  /// Show days which have no entry (monday, tuesday, wednesday, thursday and friday only)
  #[arg(long)]
  pub missing: bool,
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

  /// Start (e.g. 'now', 'today at 6am', 'yesterday at 16:30' '2021-11-30T06:00', '2 hours ago', 'yesterday at 6am') - All possible formats <https://github.com/PicoJr/htp/blob/HEAD/src/time.pest>
  #[arg(
    long,
    default_value = "now",
    value_parser = parse_time,
  )]
  pub start: DateTime<Local>,

  /// End (e.g. 'now', 'today at 6am', 'yesterday at 16:30' '2021-11-30T06:00', '2 hours ago', 'yesterday at 6am') - All possible formats <https://github.com/PicoJr/htp/blob/HEAD/src/time.pest>
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

#[derive(Parser, Debug, Clone)]
pub struct StopTimeEntry {
  /// Id of the time entry (optional, stops current if not provided)
  #[arg(long)]
  pub id: Option<TimeEntryId>,
}

#[derive(Parser, Debug, Clone)]
pub struct ContinueTimeEntry {
  /// Id of the time entry to continue (optional, continues last if not provided)
  #[arg(long)]
  pub id: Option<TimeEntryId>,
}

#[derive(Parser, Debug, Clone, Copy)]
pub struct TimeEntryDetails {
  /// Time entry ID
  pub id: TimeEntryId,
}

#[derive(Parser, Debug)]
pub struct EditTimeEntry {
  /// Time entry ID
  pub id: TimeEntryId,

  /// New project name
  #[arg(long)]
  pub project: Option<String>,

  /// New description
  #[arg(long)]
  pub description: Option<String>,

  /// New tags
  #[arg(long)]
  pub tags: Option<Vec<String>>,

  /// New start time
  #[arg(long, value_parser = parse_time)]
  pub start: Option<DateTime<Local>>,

  /// New end time
  #[arg(long, value_parser = parse_time)]
  pub end: Option<DateTime<Local>>,

  /// Toggle billable status
  #[arg(long)]
  pub toggle_billable: bool,
}

#[derive(Parser, Debug)]
pub struct CreateClient {
  /// Name of the client
  #[arg(long)]
  pub name: String,
}

pub fn output_values_json<T: Serialize>(values: &[T]) {
  for value in values {
    if let Ok(output) = serde_json::to_string_pretty(&value) {
      println!("{output}");
    }
  }
}

pub fn print_completions<G: Generator>(generator: G, cmd: &mut clap::Command) {
  generate(generator, cmd, cmd.get_name().to_owned(), &mut io::stdout());
}
