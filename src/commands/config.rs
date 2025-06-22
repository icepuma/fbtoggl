use crate::config::get_settings_file;
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

pub fn set(key: &str, value: &str) -> anyhow::Result<()> {
  let settings_path = get_settings_file()?;

  if !settings_path.exists() {
    anyhow::bail!(
      "Configuration file not found. Run 'fbtoggl config init' first."
    );
  }

  // Read the existing configuration
  let contents = fs::read_to_string(&settings_path)?;
  let mut config: toml::Value = toml::from_str(&contents)?;

  // Update the configuration
  match key {
    "api_token" => {
      if let toml::Value::Table(ref mut table) = config {
        table.insert(
          "api_token".to_owned(),
          toml::Value::String(value.to_owned()),
        );
      }
    }
    _ => {
      anyhow::bail!("Unknown configuration key: {key}. Valid keys: api_token");
    }
  }

  // Write the updated configuration back
  let updated_contents = toml::to_string_pretty(&config)?;
  fs::write(&settings_path, updated_contents)?;

  // Mask sensitive values in output
  let display_value = mask_sensitive_value(key, value);

  println!("Updated configuration:");
  println!("  {key} = {display_value}");

  Ok(())
}
