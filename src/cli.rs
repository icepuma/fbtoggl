use clap::{ArgEnum, Parser};

use crate::model::{Printer, Start};

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
  List,
}

#[derive(Parser, Debug)]
pub enum Projects {
  List,
}

#[derive(Parser, Debug)]
pub enum TimeEntries {
  #[clap(about = "List all time entries")]
  List,

  #[clap(about = "Create time entry")]
  Create(CreateTimeEntry),

  #[clap(about = "Create workday with pause")]
  CreateWorkdayWithPause(CreateWorkdayWithPause),
}

#[derive(Parser, Debug)]
pub struct CreateTimeEntry {
  #[clap(long, about = "Name of the project")]
  pub project: String,

  #[clap(long, about = "Description of the timer")]
  pub description: String,

  #[clap(long, about = "Tags")]
  pub tags: Option<Vec<String>>,

  #[clap(long, about = "Duration (in minutes)")]
  pub duration: u64,

  #[clap(long, about = "Start (now, ISO date)", default_value = "now")]
  pub start: Start,
}

#[derive(Parser, Debug)]
pub struct CreateWorkdayWithPause {
  #[clap(long, about = "Name of the project")]
  pub project: String,

  #[clap(long, about = "Description of the timer")]
  pub description: String,

  #[clap(long, about = "Duration (in hours)")]
  pub hours: f64,

  #[clap(long, about = "Start (now, ISO date)", default_value = "now")]
  pub start: Start,
}

#[derive(Parser, Debug)]
pub enum Clients {
  List,
}

pub fn output_values<T: Printer>(format: &Format, values: Vec<T>) {
  for value in values {
    output_value(format, value)
  }
}

pub fn output_value<T: Printer>(format: &Format, value: T) {
  match format {
    Format::Json => {
      if let Ok(output) = value.to_json() {
        println!("{}", output);
      }
    }
    Format::Raw => {
      if let Ok(output) = value.to_raw() {
        println!("\"{}\"", output);
      }
    }
  };
}
