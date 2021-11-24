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
  }

  Ok(())
}

pub fn list(format: &Format, client: &TogglClient) -> anyhow::Result<()> {
  let me = client.get_me()?;
  let clients = client.get_workspace_clients(me.data.default_wid)?;

  match format {
    Format::Json => output_values_json(&clients),
    Format::Raw => output_values_raw(&clients),
  }

  Ok(())
}

fn output_values_raw(values: &[Client]) {
  for client in values {
    println!("\"{}\"", client.name);
  }
}
