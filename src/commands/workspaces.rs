use super::init_client;
use crate::cli::{output_values, Format};

pub fn list(format: &Format) -> anyhow::Result<()> {
  let client = init_client()?;
  let workspaces = client.get_workspaces()?;

  output_values(format, workspaces);

  Ok(())
}
