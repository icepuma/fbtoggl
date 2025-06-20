use crate::{
  cli::{Format, output_values_json},
  client::TogglClient,
  output::{output_named_entities_raw, output_named_entities_table},
};

pub fn list(
  debug: bool,
  format: &Format,
  client: &TogglClient,
) -> anyhow::Result<()> {
  let workspaces = client.get_workspaces(debug)?;

  match format {
    Format::Json => output_values_json(&workspaces),
    Format::Raw => output_named_entities_raw(&workspaces),
    Format::Table => output_named_entities_table(&workspaces, "Name"),
  }

  Ok(())
}
