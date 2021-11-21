use super::init_client;
use crate::cli::{output_value, output_values, CreateClient, Format};

pub fn create(format: &Format, create_client: &CreateClient) -> anyhow::Result<()> {
  let client = init_client()?;
  let me = client.get_me()?;

  let data = client.create_client(&create_client.name, me.data.default_wid)?;

  output_value(format, data.data);

  Ok(())
}

pub fn list(format: &Format) -> anyhow::Result<()> {
  let client = init_client()?;
  let me = client.get_me()?;
  let clients = client.get_workspace_clients(me.data.default_wid)?;

  output_values(format, clients);

  Ok(())
}
