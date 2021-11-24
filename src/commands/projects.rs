use crate::{
  cli::{output_values_json, Format},
  client::TogglClient,
  model::Project,
};

pub fn list(format: &Format, client: &TogglClient) -> anyhow::Result<()> {
  let me = client.get_me()?;
  let workspace_projects =
    client.get_workspace_projects(me.data.default_wid)?;

  match format {
    Format::Json => output_values_json(&workspace_projects),
    Format::Raw => output_values_raw(&workspace_projects),
  }

  Ok(())
}

fn output_values_raw(values: &[Project]) {
  for project in values {
    println!("\"{}\"", project.name);
  }
}
