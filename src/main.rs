//! fbtoggl - A terminal client for the Toggl Track API
//!
//! This application provides a command-line interface to interact with
//! Toggl Track's time tracking service.

use crate::cli::{Clients, Options, SubCommand, TimeEntries};
use crate::config::init_settings_file;
use clap::Parser;
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
  let format = options.format;
  let debug = options.debug;

  match options.subcommand {
    SubCommand::Init => init_settings_file()?,
    SubCommand::Settings(action) => match action {
      Settings::Init => init_settings_file()?,
    },
    SubCommand::Projects(action) => match action {
      Projects::List(list_projects) => {
        let client = init_client()?;

        commands::projects::list(
          debug,
          list_projects.include_archived,
          &format,
          &client,
        )?;
      }
    },
    SubCommand::Workspaces(_action) => {
      let client = init_client()?;

      commands::workspaces::list(debug, &format, &client)?;
    }

    SubCommand::TimeEntries(action) => match action {
      TimeEntries::Create(time_entry) => {
        let client = init_client()?;
        commands::time_entries::create(debug, &format, &time_entry, &client)?;
      }
      TimeEntries::List(list_time_entries) => {
        let client = init_client()?;
        commands::time_entries::list(
          debug,
          &format,
          &list_time_entries.range,
          list_time_entries.missing,
          &client,
        )?;
      }
      TimeEntries::Start(time_entry) => {
        let client = init_client()?;
        commands::time_entries::start(debug, &format, &time_entry, &client)?;
      }
      TimeEntries::Stop(time_entry) => {
        let client = init_client()?;
        commands::time_entries::stop(debug, &format, &time_entry, &client)?;
      }
      TimeEntries::Delete(time_entry) => {
        let client = init_client()?;
        commands::time_entries::delete(debug, &format, &time_entry, &client)?;
      }
      TimeEntries::Details(time_entry) => {
        let client = init_client()?;
        commands::time_entries::details(debug, &format, &time_entry, &client)?;
      }
    },

    SubCommand::Clients(action) => match action {
      Clients::Create(create_client) => {
        let client = init_client()?;
        commands::clients::create(debug, &format, &create_client, &client)?;
      }
      Clients::List(list_clients) => {
        let client = init_client()?;
        commands::clients::list(
          debug,
          list_clients.include_archived,
          &format,
          &client,
        )?;
      }
    },

    SubCommand::Reports(action) => match action {
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
    },
  }

  Ok(())
}
