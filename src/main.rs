use crate::cli::{Options, SubCommand, TimeEntries};
use crate::config::init_settings_file;
use clap::Parser;

mod cli;
mod client;
mod commands;
mod config;
mod model;

fn main() -> anyhow::Result<()> {
  let options = Options::parse();
  let format = options.format;

  match options.subcommand {
    SubCommand::Init => init_settings_file()?,
    SubCommand::Projects(_action) => commands::projects::list(&format)?,
    SubCommand::Workspaces(_action) => commands::workspaces::list(&format)?,

    SubCommand::TimeEntries(action) => match action {
      TimeEntries::CreateWorkdayWithPause(time_entry) => commands::time_entries::create_workday_with_pause(&time_entry)?,

      TimeEntries::Create(time_entry) => commands::time_entries::create(&format, &time_entry)?,
      TimeEntries::List => commands::time_entries::list(&format)?,
    },

    SubCommand::Clients(_action) => commands::clients::list(&format)?,
  }

  Ok(())
}
