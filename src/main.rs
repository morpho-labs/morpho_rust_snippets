use alloy::transports::http::reqwest::Url;
use eyre::Result;
mod market;
mod vault;

#[tokio::main]
async fn main() -> Result<()> {
    let rpc_url: Url =
        "https://eth-mainnet.g.alchemy.com/v2/YOTcUDy_k90iZVmkAtMgxzgWOtcc_z3J".parse()?;

    market::get_market(rpc_url.clone()).await?;
    market::read_events_with_get_logs(rpc_url.clone()).await?;
    vault::get_vault_data(rpc_url.clone()).await?;
    vault::read_vaults(rpc_url.clone()).await?;
    vault::get_vault_activity(rpc_url.clone()).await?;

    Ok(())
}
