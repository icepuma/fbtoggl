use crate::{
  cli::{Format, output_values_json},
  client::TogglClient,
  output::{output_named_entities_raw, output_named_entities_table},
};
use anyhow::Context;

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

pub fn create(
  debug: bool,
  format: &Format,
  name: &str,
  client_name: Option<&str>,
  billable: bool,
  color: Option<&str>,
  client: &TogglClient,
) -> anyhow::Result<()> {
  let me = client.get_me(debug)?;
  let workspace_id = me.default_workspace_id;

  // If client name is provided, look up the client ID
  let client_id = if let Some(client_name) = client_name {
    let clients = client
      .get_workspace_clients(debug, false, workspace_id)?
      .context("Failed to get clients")?;

    clients
      .iter()
      .find(|c| c.name.eq_ignore_ascii_case(client_name))
      .map(|c| c.id)
  } else {
    None
  };

  let project = client.create_project(
    debug,
    name,
    workspace_id,
    client_id,
    billable,
    color,
  )?;

  println!("Created project: {}", project.name);

  match format {
    Format::Json => output_values_json(&[project]),
    Format::Raw => output_named_entities_raw(&[project]),
    Format::Table => output_named_entities_table(&[project], "Project"),
  }

  Ok(())
}
