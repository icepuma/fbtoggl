use crate::client::TogglClient;
use crate::config::read_settings;

pub mod clients;
pub mod projects;
pub mod time_entries;
pub mod workspaces;

fn init_client() -> anyhow::Result<TogglClient> {
    let settings = read_settings()?;

    TogglClient::new(settings.api_token)
}
