//! fbtoggl - A terminal client for the Toggl Track API
//!
//! This application provides a command-line interface to interact with
//! Toggl Track's time tracking service.

use crate::cli::{Format, Options, SubCommand};
use crate::config::init_settings_file;
use crate::types::TimeEntryId;
use clap::{CommandFactory, Parser};
use client::init_client;
use report_client::init_report_client;

mod cli;
mod client;
mod commands;
mod common;
mod config;
mod error;
mod http_client;
mod model;
mod output;
mod report_client;
mod types;

#[cfg(test)]
mod client_tests;

fn main() -> anyhow::Result<()> {
  let options = Options::parse();

  let format = options.format;
  let debug = options.debug;

  if let Some(subcommand) = options.subcommand {
    execute_subcommand(subcommand, debug, &format)?;
  } else {
    eprintln!("Error: A subcommand is required");
    std::process::exit(1);
  }

  Ok(())
}

fn execute_subcommand(
  subcommand: SubCommand,
  debug: bool,
  format: &Format,
) -> anyhow::Result<()> {
  match subcommand {
    SubCommand::Init => init_settings_file()?,

    // Time entry commands
    SubCommand::Start(time_entry) => {
      handle_time_entry_start(debug, format, &time_entry)?;
    }
    SubCommand::Stop(stop_entry) => {
      handle_time_entry_stop(debug, format, &stop_entry)?;
    }
    SubCommand::Continue(continue_entry) => {
      handle_time_entry_continue(debug, format, &continue_entry)?;
    }
    SubCommand::Current => handle_time_entry_current(debug, format)?,
    SubCommand::Add(time_entry) => {
      handle_time_entry_add(debug, format, &time_entry)?;
    }
    SubCommand::Log(list_time_entries) => {
      handle_time_entry_log(debug, format, &list_time_entries)?;
    }
    SubCommand::Show(details) => {
      handle_time_entry_show(debug, format, details)?;
    }
    SubCommand::Edit(edit_entry) => {
      handle_time_entry_edit(debug, format, &edit_entry)?;
    }
    SubCommand::Delete { id } => handle_time_entry_delete(debug, format, id)?,

    // Report commands
    SubCommand::Report(report_options) => handle_report(debug, report_options)?,
    SubCommand::Summary(summary_options) => {
      handle_summary(debug, format, summary_options)?;
    }

    // Resource management commands
    SubCommand::Workspace(action) => handle_workspace(debug, format, action)?,
    SubCommand::Project(action) => handle_project(debug, format, &action)?,
    SubCommand::Client(action) => handle_client(debug, format, &action)?,

    // Configuration commands
    SubCommand::Config(action) => handle_config(action)?,

    // Completions command
    SubCommand::Completions { shell } => {
      let mut cmd = Options::command();
      cli::print_completions(shell, &mut cmd);
    }
  }
  Ok(())
}

fn handle_time_entry_start(
  debug: bool,
  format: &Format,
  time_entry: &cli::StartTimeEntry,
) -> anyhow::Result<()> {
  let client = init_client()?;
  commands::time_entries::start(debug, format, time_entry, &client)
}

fn handle_time_entry_stop(
  debug: bool,
  format: &Format,
  stop_entry: &cli::StopTimeEntry,
) -> anyhow::Result<()> {
  let client = init_client()?;
  if stop_entry.id.is_some() {
    commands::time_entries::stop(debug, format, stop_entry, &client)
  } else {
    commands::time_entries::stop_current(debug, format, &client)
  }
}

fn handle_time_entry_continue(
  debug: bool,
  format: &Format,
  continue_entry: &cli::ContinueTimeEntry,
) -> anyhow::Result<()> {
  let client = init_client()?;
  commands::time_entries::continue_timer(
    debug,
    format,
    continue_entry.id,
    &client,
  )
}

fn handle_time_entry_current(
  debug: bool,
  format: &Format,
) -> anyhow::Result<()> {
  let client = init_client()?;
  commands::time_entries::current(debug, format, &client)
}

fn handle_time_entry_add(
  debug: bool,
  format: &Format,
  time_entry: &cli::CreateTimeEntry,
) -> anyhow::Result<()> {
  let client = init_client()?;
  commands::time_entries::create(debug, format, time_entry, &client)
}

fn handle_time_entry_log(
  debug: bool,
  format: &Format,
  list_time_entries: &cli::ListTimeEntries,
) -> anyhow::Result<()> {
  let client = init_client()?;
  commands::time_entries::list(
    debug,
    format,
    &list_time_entries.range,
    list_time_entries.missing,
    &client,
  )
}

fn handle_time_entry_show(
  debug: bool,
  format: &Format,
  details: cli::TimeEntryDetails,
) -> anyhow::Result<()> {
  let client = init_client()?;
  commands::time_entries::details(debug, format, details, &client)
}

fn handle_time_entry_edit(
  debug: bool,
  format: &Format,
  edit_entry: &cli::EditTimeEntry,
) -> anyhow::Result<()> {
  let client = init_client()?;
  commands::time_entries::edit(debug, format, edit_entry, &client)
}

fn handle_time_entry_delete(
  debug: bool,
  format: &Format,
  id: TimeEntryId,
) -> anyhow::Result<()> {
  let client = init_client()?;
  let time_entry = cli::TimeEntryDetails { id };
  commands::time_entries::delete(debug, format, time_entry, &client)
}

fn handle_report(
  debug: bool,
  report_options: cli::ReportOptions,
) -> anyhow::Result<()> {
  let client = init_client()?;
  let report_client = init_report_client()?;
  commands::reports::detailed(
    debug,
    &client,
    &report_options.range,
    &report_client,
  )
}

fn handle_summary(
  debug: bool,
  format: &Format,
  summary_options: cli::SummaryOptions,
) -> anyhow::Result<()> {
  let client = init_client()?;
  commands::reports::summary(debug, &client, &summary_options.range, format)
}

fn handle_workspace(
  debug: bool,
  format: &Format,
  action: cli::Workspace,
) -> anyhow::Result<()> {
  let client = init_client()?;
  match action {
    cli::Workspace::List => commands::workspaces::list(debug, format, &client),
  }
}

fn handle_project(
  debug: bool,
  format: &Format,
  action: &cli::Project,
) -> anyhow::Result<()> {
  let client = init_client()?;
  match action {
    cli::Project::List { all } => {
      commands::projects::list(debug, *all, format, &client)
    }
    cli::Project::Create {
      name,
      client: client_name,
      billable,
      color,
    } => commands::projects::create(
      debug,
      format,
      name,
      client_name.as_deref(),
      *billable,
      color.as_deref(),
      &client,
    ),
  }
}

fn handle_client(
  debug: bool,
  format: &Format,
  action: &cli::Client,
) -> anyhow::Result<()> {
  let client = init_client()?;
  match action {
    cli::Client::List { all } => {
      commands::clients::list(debug, *all, format, &client)
    }
    cli::Client::Create { name } => {
      let create_client = cli::CreateClient { name: name.clone() };
      commands::clients::create(debug, format, &create_client, &client)
    }
  }
}

fn handle_config(action: cli::Config) -> anyhow::Result<()> {
  match action {
    cli::Config::Init => init_settings_file(),
    cli::Config::Show => commands::config::show(),
    cli::Config::Set { key, value } => commands::config::set(&key, &value),
  }
}
