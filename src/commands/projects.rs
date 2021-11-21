use super::init_client;
use crate::cli::{output_values, Format};

pub fn list(format: &Format) -> anyhow::Result<()> {
  let client = init_client()?;
  let me = client.get_me()?;
  let workspace_projects =
    client.get_workspace_projects(me.data.default_wid)?;

  output_values(format, workspace_projects);

  Ok(())
}
