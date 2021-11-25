use colored::Colorize;
use term_table::{row::Row, table_cell::TableCell, Table, TableStyle};

use crate::{
  cli::{output_values_json, CreateClient, Format},
  client::TogglClient,
  model::Client,
};

pub fn create(
  format: &Format,
  create_client: &CreateClient,
  client: &TogglClient,
) -> anyhow::Result<()> {
  let me = client.get_me()?;

  let data = client.create_client(&create_client.name, me.data.default_wid)?;

  match format {
    Format::Json => output_values_json(&[data.data]),
    Format::Raw => output_values_raw(&[data.data]),
    Format::Table => output_values_table(&[data.data]),
  }

  Ok(())
}

pub fn list(format: &Format, client: &TogglClient) -> anyhow::Result<()> {
  let me = client.get_me()?;
  let clients = client.get_workspace_clients(me.data.default_wid)?;

  match format {
    Format::Json => output_values_json(&clients),
    Format::Raw => output_values_raw(&clients),
    Format::Table => output_values_table(&clients),
  }

  Ok(())
}

fn output_values_raw(values: &[Client]) {
  for client in values {
    println!("\"{}\"", client.name);
  }
}

fn output_values_table(values: &[Client]) {
  let mut table = Table::new();
  table.style = TableStyle::thin();

  let header = Row::new(vec![
    TableCell::new("ID".bold().white()),
    TableCell::new("Name".bold().white()),
  ]);

  table.add_row(header);

  for client in values {
    let row = Row::new(vec![
      TableCell::new(client.id),
      TableCell::new(&client.name),
    ]);

    table.add_row(row);
  }

  println!("{}", table.render());
}
