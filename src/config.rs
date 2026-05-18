use std::path::Path;

use config::Config;
use dialoguer::{Confirm, Password};
use serde::{Deserialize, Serialize};

use crate::cli::APP_NAME;
use crate::types::ApiToken;

#[derive(Debug, Deserialize)]
pub struct Settings {
  pub api_token: ApiToken,
}

/// Used only when writing the settings file (stores raw string on disk).
#[derive(Serialize)]
struct SettingsOnDisk<'a> {
  api_token: &'a str,
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

  let content = toml::to_string_pretty(&SettingsOnDisk {
    api_token: &api_token,
  })?;

  write_secure(path, &content)?;

  println!("Wrote settings file to {}", path.display());

  Ok(())
}

/// Write a config file with mode 0600 on Unix; atomic via tempfile + rename.
pub fn write_secure(path: &Path, content: &str) -> anyhow::Result<()> {
  let parent = path.parent().ok_or_else(|| {
    anyhow::anyhow!("Config path has no parent: {}", path.display())
  })?;
  std::fs::create_dir_all(parent)?;

  let tmp = parent.join(format!(
    ".{}.tmp",
    path.file_name().and_then(|s| s.to_str()).unwrap_or("settings")
  ));

  #[cfg(unix)]
  {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;

    let mut f = std::fs::OpenOptions::new()
      .write(true)
      .create(true)
      .truncate(true)
      .mode(0o600)
      .open(&tmp)?;
    f.write_all(content.as_bytes())?;
    f.sync_all()?;
  }

  #[cfg(not(unix))]
  std::fs::write(&tmp, content)?;

  std::fs::rename(&tmp, path)?;
  Ok(())
}

pub fn read_settings() -> anyhow::Result<Settings> {
  let xdg_dirs = xdg::BaseDirectories::with_prefix(APP_NAME);
  let settings_file =
    xdg_dirs.find_config_file("settings.toml").ok_or_else(|| {
      anyhow::anyhow!(
        "Settings file not found. Run 'fbtoggl config init' to create one."
      )
    })?;

  let settings = Config::builder()
    .add_source(config::File::from(settings_file))
    .build()?;

  Ok(settings.try_deserialize()?)
}

pub fn get_settings_file() -> anyhow::Result<std::path::PathBuf> {
  let xdg_dirs = xdg::BaseDirectories::with_prefix(APP_NAME);
  xdg_dirs.find_config_file("settings.toml").ok_or_else(|| {
    anyhow::anyhow!(
      "Settings file not found. Run 'fbtoggl config init' to create one."
    )
  })
}
