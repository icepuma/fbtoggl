use colored::Colorize;
use term_table::{row::Row, table_cell::TableCell, Table, TableStyle};

use crate::{
  cli::{output_values_json, Format},
  client::TogglClient,
  model::Project,
};

pub fn list(format: &Format, client: &TogglClient) -> anyhow::Result<()> {
  let me = client.get_me()?;
  let workspace_projects =
    client.get_workspace_projects(me.data.default_wid)?;

  match format {
    Format::Json => output_values_json(&workspace_projects),
    Format::Raw => output_values_raw(&workspace_projects),
    Format::Table => output_values_table(&workspace_projects),
  }

  Ok(())
}

fn output_values_raw(values: &[Project]) {
  for project in values {
    println!("\"{}\"", project.name);
  }
}

fn output_values_table(values: &[Project]) {
  let mut table = Table::new();
  table.style = TableStyle::thin();

  let header = Row::new(vec![
    TableCell::new("ID".bold().white()),
    TableCell::new("Name".bold().white()),
  ]);

  table.add_row(header);

  for project in values {
    let row = Row::new(vec![
      TableCell::new(project.id),
      TableCell::new(&project.name),
    ]);

    table.add_row(row);
  }

  println!("{}", table.render());
}
