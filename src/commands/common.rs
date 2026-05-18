use crate::model::Project;
use anyhow::anyhow;

pub fn find_project_by_name<'a>(
  projects: &'a [Project],
  project_name: &str,
) -> anyhow::Result<&'a Project> {
  // 1. Exact match.
  if let Some(p) = projects.iter().find(|p| p.name == project_name) {
    return Ok(p);
  }

  // 2. Case-insensitive match — common typo class (acme vs Acme).
  if let Some(p) = projects
    .iter()
    .find(|p| p.name.eq_ignore_ascii_case(project_name))
  {
    return Ok(p);
  }

  // 3. No match: list close candidates by Levenshtein distance, then full list.
  let needle = project_name.to_lowercase();
  let mut ranked: Vec<(usize, &str)> = projects
    .iter()
    .map(|p| {
      (
        levenshtein(&p.name.to_lowercase(), &needle),
        p.name.as_str(),
      )
    })
    .collect();
  ranked.sort_by_key(|(d, _)| *d);

  let suggestions: Vec<&str> = ranked
    .iter()
    .filter(|(d, _)| *d <= 3)
    .take(3)
    .map(|(_, n)| *n)
    .collect();

  let all_names: Vec<&str> = projects.iter().map(|p| p.name.as_str()).collect();

  Err(if suggestions.is_empty() {
    anyhow!(
      "No project named '{project_name}'. Available: {}",
      all_names.join(", ")
    )
  } else {
    anyhow!(
      "No project named '{project_name}'. Did you mean: {}? \
       (Available: {})",
      suggestions.join(", "),
      all_names.join(", ")
    )
  })
}

/// Classic iterative two-row Levenshtein distance, case-sensitive.
/// Inputs are expected to be lowercased by the caller.
fn levenshtein(a: &str, b: &str) -> usize {
  let a: Vec<char> = a.chars().collect();
  let b: Vec<char> = b.chars().collect();

  if a.is_empty() {
    return b.len();
  }
  if b.is_empty() {
    return a.len();
  }

  // Two rolling rows of the DP matrix; index-free via iterators + push.
  let mut prev: Vec<usize> = (0..=b.len()).collect();

  for (i, ca) in a.iter().enumerate() {
    let mut curr: Vec<usize> = Vec::with_capacity(b.len().saturating_add(1));
    curr.push(i.saturating_add(1));

    for (j, cb) in b.iter().enumerate() {
      let cost: usize = usize::from(ca != cb);
      // del = prev[j+1] + 1, ins = curr[j] + 1, sub = prev[j] + cost.
      // All three reads are guaranteed in-range by the loop invariants;
      // .get() makes that explicit to the indexing lint.
      let del = prev
        .get(j.saturating_add(1))
        .copied()
        .unwrap_or(0)
        .saturating_add(1);
      let ins = curr.last().copied().unwrap_or(0).saturating_add(1);
      let sub = prev.get(j).copied().unwrap_or(0).saturating_add(cost);
      curr.push(del.min(ins).min(sub));
    }

    prev = curr;
  }

  prev.last().copied().unwrap_or(0)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "Test code can panic on failure")]
mod tests {
  use super::*;
  use crate::types::{ProjectId, WorkspaceId};

  fn project(name: &str) -> Project {
    Project {
      id: ProjectId::new(1),
      name: name.to_owned(),
      workspace_id: WorkspaceId::new(1),
      status: None,
      active: None,
      client_id: None,
    }
  }

  #[test]
  fn exact_match_wins() {
    let projects = vec![project("Acme"), project("Beta")];
    let p = find_project_by_name(&projects, "Acme").unwrap();
    assert_eq!(p.name, "Acme");
  }

  #[test]
  fn case_insensitive_fallback() {
    let projects = vec![project("Acme")];
    let p = find_project_by_name(&projects, "acme").unwrap();
    assert_eq!(p.name, "Acme");
  }

  #[test]
  fn typo_suggests_did_you_mean() {
    let projects = vec![project("Acme"), project("Beta")];
    let err = find_project_by_name(&projects, "akme").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Did you mean: Acme"), "got: {msg}");
  }

  #[test]
  fn no_close_match_lists_all() {
    let projects = vec![project("Acme"), project("Beta")];
    let err = find_project_by_name(&projects, "xyzzyz").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Available: Acme, Beta"), "got: {msg}");
    assert!(!msg.contains("Did you mean"), "got: {msg}");
  }
}
