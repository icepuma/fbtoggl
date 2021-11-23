use crate::{
  cli::{output_values, Format},
  client::TogglClient,
};

pub fn list(format: &Format, client: &TogglClient) -> anyhow::Result<()> {
  let workspaces = client.get_workspaces()?;

  output_values(format, workspaces);

  Ok(())
}
