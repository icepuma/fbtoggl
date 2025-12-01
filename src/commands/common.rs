use crate::model::Project;
use anyhow::anyhow;
use std::collections::HashMap;

pub fn find_project_by_name<'a>(
  projects: &'a [Project],
  project_name: &str,
) -> anyhow::Result<&'a Project> {
  // Create HashMap for O(1) lookup
  let projects_by_name: HashMap<&str, &Project> =
    projects.iter().map(|p| (p.name.as_str(), p)).collect();

  projects_by_name
    .get(project_name)
    .ok_or_else(|| anyhow!("Cannot find project='{project_name}'"))
    .copied()
}
