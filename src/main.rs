//! fbtoggl - A terminal client for the Toggl Track API
//!
//! This application provides a command-line interface to interact with
//! Toggl Track's time tracking service.

use crate::cli::{Clients, Format, Options, SubCommand, TimeEntries};
use crate::config::init_settings_file;
use clap::{CommandFactory, Parser};
use cli::{Projects, Reports, Settings};
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

  // Handle completion generation if requested
  if let Some(shell) = options.completions {
    let mut cmd = Options::command();
    cli::print_completions(shell, &mut cmd);
    return Ok(());
  }

  let format = options.format;
  let debug = options.debug;

  if let Some(subcommand) = options.subcommand {
    execute_subcommand(subcommand, debug, &format)?;
  } else {
    eprintln!(
      "Error: A subcommand is required when not generating completions"
    );
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
    SubCommand::Settings(action) => handle_settings(action)?,
    SubCommand::Projects(action) => handle_projects(action, debug, format)?,
    SubCommand::Workspaces(_action) => handle_workspaces(debug, format)?,
    SubCommand::TimeEntries(action) => {
      handle_time_entries(action, debug, format)?;
    }
    SubCommand::Clients(action) => handle_clients(action, debug, format)?,
    SubCommand::Reports(action) => handle_reports(action, debug)?,
  }
  Ok(())
}

fn handle_settings(action: Settings) -> anyhow::Result<()> {
  match action {
    Settings::Init => init_settings_file()?,
  }
  Ok(())
}

fn handle_projects(
  action: Projects,
  debug: bool,
  format: &Format,
) -> anyhow::Result<()> {
  match action {
    Projects::List(list_projects) => {
      let client = init_client()?;
      commands::projects::list(
        debug,
        list_projects.include_archived,
        format,
        &client,
      )?;
    }
  }
  Ok(())
}

fn handle_workspaces(debug: bool, format: &Format) -> anyhow::Result<()> {
  let client = init_client()?;
  commands::workspaces::list(debug, format, &client)?;
  Ok(())
}

fn handle_time_entries(
  action: TimeEntries,
  debug: bool,
  format: &Format,
) -> anyhow::Result<()> {
  let client = init_client()?;

  match action {
    TimeEntries::Create(time_entry) => {
      commands::time_entries::create(debug, format, &time_entry, &client)?;
    }
    TimeEntries::List(list_time_entries) => {
      commands::time_entries::list(
        debug,
        format,
        &list_time_entries.range,
        list_time_entries.missing,
        &client,
      )?;
    }
    TimeEntries::Start(time_entry) => {
      commands::time_entries::start(debug, format, &time_entry, &client)?;
    }
    TimeEntries::Stop(time_entry) => {
      commands::time_entries::stop(debug, format, &time_entry, &client)?;
    }
    TimeEntries::Delete(time_entry) => {
      commands::time_entries::delete(debug, format, &time_entry, &client)?;
    }
    TimeEntries::Details(time_entry) => {
      commands::time_entries::details(debug, format, &time_entry, &client)?;
    }
  }
  Ok(())
}

fn handle_clients(
  action: Clients,
  debug: bool,
  format: &Format,
) -> anyhow::Result<()> {
  let client = init_client()?;

  match action {
    Clients::Create(create_client) => {
      commands::clients::create(debug, format, &create_client, &client)?;
    }
    Clients::List(list_clients) => {
      commands::clients::list(
        debug,
        list_clients.include_archived,
        format,
        &client,
      )?;
    }
  }
  Ok(())
}

fn handle_reports(action: Reports, debug: bool) -> anyhow::Result<()> {
  match action {
    Reports::Detailed(detailed) => {
      let client = init_client()?;
      let report_client = init_report_client()?;
      commands::reports::detailed(
        debug,
        &client,
        &detailed.range,
        &report_client,
      )?;
    }
  }
  Ok(())
}
