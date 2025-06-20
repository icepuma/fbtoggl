use std::path::Path;

use config::Config;
use dialoguer::{Confirm, Password};
use serde::{Deserialize, Serialize};

use crate::cli::APP_NAME;

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
  pub api_token: String,
}

pub fn init_settings_file() -> anyhow::Result<()> {
  let xdg_dirs = xdg::BaseDirectories::with_prefix(APP_NAME);

  // Check if config file already exists
  if let Some(existing_file) = xdg_dirs.find_config_file("settings.toml") {
    if Confirm::new()
      .with_prompt("Override settings.toml file?")
      .interact()?
    {
      println!("Override settings file {}", existing_file.display());
      write_config_file(&existing_file)?;
    } else {
      println!("Do nothing!");
    }
  } else {
    // Create new config file
    let settings_file = xdg_dirs.place_config_file("settings.toml")?;
    write_config_file(&settings_file)?;
  }

  Ok(())
}

fn write_config_file(path: &Path) -> anyhow::Result<()> {
  let api_token = Password::new()
    .with_prompt("New API token")
    .allow_empty_password(false)
    .interact()?;

  let settings = Settings { api_token };
  let content = toml::to_string_pretty(&settings)?;

  std::fs::write(path, content)?;

  println!("Wrote settings file to {}", path.display());

  Ok(())
}

pub fn read_settings() -> anyhow::Result<Settings> {
  let xdg_dirs = xdg::BaseDirectories::with_prefix(APP_NAME);
  let settings_file =
    xdg_dirs.find_config_file("settings.toml").ok_or_else(|| {
      anyhow::anyhow!(
        "Settings file not found. Run 'fbtoggl settings init' to create one."
      )
    })?;

  let settings = Config::builder()
    .add_source(config::File::from(settings_file))
    .build()?;

  Ok(settings.try_deserialize()?)
}
