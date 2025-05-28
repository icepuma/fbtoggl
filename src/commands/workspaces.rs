use colored::Colorize;
use term_table::{Table, TableStyle, row::Row, table_cell::TableCell};

use crate::{
  cli::{Format, output_values_json},
  client::TogglClient,
  model::Workspace,
};

pub fn list(
  debug: bool,
  format: &Format,
  client: &TogglClient,
) -> anyhow::Result<()> {
  let workspaces = client.get_workspaces(debug)?;

  match format {
    Format::Json => output_values_json(&workspaces),
    Format::Raw => output_values_raw(&workspaces),
    Format::Table => output_values_table(&workspaces),
  }

  Ok(())
}

fn output_values_raw(values: &[Workspace]) {
  for workspace in values {
    println!("\"{}\"", workspace.name);
  }
}

fn output_values_table(values: &[Workspace]) {
  let mut table = Table::new();
  table.style = TableStyle::thin();

  let header = Row::new(vec![
    TableCell::new("ID".bold().white()),
    TableCell::new("Name".bold().white()),
  ]);

  table.add_row(header);

  for workspace in values {
    let row = Row::new(vec![
      TableCell::new(workspace.id),
      TableCell::new(&workspace.name),
    ]);

    table.add_row(row);
  }

  println!("{}", table.render());
}
