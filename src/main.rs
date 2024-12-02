use alloy::transports::http::reqwest::Url;
use eyre::Result;
mod api;
mod arithmetic;
mod market;
mod vault;
#[tokio::main]
async fn main() -> Result<()> {
    let rpc_url: Url =
        "https://eth-mainnet.g.alchemy.com/v2/YOTcUDy_k90iZVmkAtMgxzgWOtcc_z3J".parse()?;

    let wss_url = "wss://eth-mainnet.g.alchemy.com/v2/YOTcUDy_k90iZVmkAtMgxzgWOtcc_z3J";
    market::retrieve_market_info(rpc_url.clone()).await?;
    //market::subscribe_to_event_logs(wss_url).await?;
    market::retrieve_events_with_logs(rpc_url.clone()).await?;
    vault::retrieve_vault_details(rpc_url.clone()).await?;
    vault::fetch_vaults(rpc_url.clone()).await?;
    vault::fetch_vault_activity_details(rpc_url.clone()).await?;

    Ok(())
}
