use crate::{
  cli::{output_values, Format},
  client::TogglClient,
};

pub fn list(format: &Format, client: &TogglClient) -> anyhow::Result<()> {
  let me = client.get_me()?;
  let workspace_projects =
    client.get_workspace_projects(me.data.default_wid)?;

  output_values(format, workspace_projects);

  Ok(())
}
