use crate::{
  cli::{Format, output_values_json},
  client::TogglClient,
  output::{output_named_entities_raw, output_named_entities_table},
};

pub fn list(
  debug: bool,
  include_archived: bool,
  format: &Format,
  client: &TogglClient,
) -> anyhow::Result<()> {
  let me = client.get_me(debug)?;
  let workspace_projects = client.get_workspace_projects(
    debug,
    include_archived,
    me.default_workspace_id,
  )?;

  if workspace_projects.is_empty() {
    println!("No entries found!");
  } else {
    match format {
      Format::Json => output_values_json(&workspace_projects),
      Format::Raw => output_named_entities_raw(&workspace_projects),
      Format::Table => output_named_entities_table(&workspace_projects, "Name"),
    }
  }

  Ok(())
}
