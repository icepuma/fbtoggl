use crate::config::{get_settings_file, write_secure};
use dialoguer::Password;
use std::fs;

fn mask_sensitive_value(key: &str, value: &str) -> String {
  match key {
    "api_token" => {
      if value.len() > 8 {
        format!(
          "{}...{}",
          &value[..4],
          &value[value.len().saturating_sub(4)..]
        )
      } else {
        "*****".to_owned()
      }
    }
    _ => value.to_owned(),
  }
}

pub fn show() -> anyhow::Result<()> {
  let settings_path = get_settings_file()?;

  if settings_path.exists() {
    let contents = fs::read_to_string(&settings_path)?;
    let config: toml::Value = toml::from_str(&contents)?;

    println!("Configuration file: {}", settings_path.display());

    // Display configuration with masked sensitive values
    if let toml::Value::Table(table) = config {
      for (key, value) in table {
        let display_value = match &value {
          toml::Value::String(s) => mask_sensitive_value(&key, s),
          _ => value.to_string(),
        };
        println!("{key} = \"{display_value}\"");
      }
    }
  } else {
    println!(
      "No configuration file found. Run 'fbtoggl config init' to create one."
    );
  }

  Ok(())
}

pub fn set(key: &str, value: Option<&str>) -> anyhow::Result<()> {
  let settings_path = get_settings_file()?;

  if !settings_path.exists() {
    anyhow::bail!(
      "Configuration file not found. Run 'fbtoggl config init' first."
    );
  }

  // For sensitive keys, never accept the value from CLI (it would land in
  // shell history). Prompt securely instead.
  let resolved_value = match key {
    "api_token" => {
      if value.is_some() {
        anyhow::bail!(
          "api_token must not be passed on the command line (shell history exposure). \
           Run 'fbtoggl config set api_token' without a value to be prompted."
        );
      }
      Password::new()
        .with_prompt("New API token")
        .allow_empty_password(false)
        .interact()?
    }
    _ => {
      anyhow::bail!("Unknown configuration key: {key}. Valid keys: api_token");
    }
  };

  let contents = fs::read_to_string(&settings_path)?;
  let mut config: toml::Value = toml::from_str(&contents)?;

  if let toml::Value::Table(ref mut table) = config {
    table.insert(key.to_owned(), toml::Value::String(resolved_value.clone()));
  }

  let updated_contents = toml::to_string_pretty(&config)?;
  write_secure(&settings_path, &updated_contents)?;

  println!("Updated configuration:");
  println!("  {} = {}", key, mask_sensitive_value(key, &resolved_value));

  Ok(())
}
