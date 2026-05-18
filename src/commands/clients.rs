use crate::{
  cli::{CreateClient, Format, output_values_json},
  client::TogglClient,
  output::{output_named_entities_raw, output_named_entities_table},
};

pub fn create(
  format: &Format,
  create_client: &CreateClient,
  client: &TogglClient,
) -> anyhow::Result<()> {
  let me = client.get_me()?;

  let data =
    client.create_client(&create_client.name, me.default_workspace_id)?;

  match format {
    Format::Json => output_values_json(&[data]),
    Format::Raw => output_named_entities_raw(&[data]),
    Format::Table => output_named_entities_table(&[data], "Name"),
  }

  Ok(())
}

pub fn list(
  include_archived: bool,
  format: &Format,
  client: &TogglClient,
) -> anyhow::Result<()> {
  let me = client.get_me()?;

  if let Ok(Some(clients)) =
    client.get_workspace_clients(include_archived, me.default_workspace_id)
  {
    match format {
      Format::Json => output_values_json(&clients),
      Format::Raw => output_named_entities_raw(&clients),
      Format::Table => output_named_entities_table(&clients, "Name"),
    }
  } else {
    println!("No entries found!");
  }

  Ok(())
}
