use crate::{
  cli::{output_values_json, Format},
  client::TogglClient,
  model::Workspace,
};

pub fn list(format: &Format, client: &TogglClient) -> anyhow::Result<()> {
  let workspaces = client.get_workspaces()?;

  match format {
    Format::Json => output_values_json(&workspaces),
    Format::Raw => output_values_raw(&workspaces),
  }

  Ok(())
}

fn output_values_raw(values: &[Workspace]) {
  for workspace in values {
    println!("\"{}\"", workspace.name);
  }
}
