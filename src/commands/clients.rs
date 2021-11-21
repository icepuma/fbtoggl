use super::init_client;
use crate::cli::{output_values, Format};

pub fn list(format: &Format) -> anyhow::Result<()> {
    let client = init_client()?;
    let me = client.get_me()?;
    let clients = client.get_workspace_clients(me.data.default_wid)?;

    output_values(format, clients);

    Ok(())
}
