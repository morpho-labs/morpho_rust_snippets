use alloy::{
    primitives::{address, B256},
    providers::ProviderBuilder,
    sol,
};
use eyre::Result;

mod market;
mod vault;

// Code gen
sol!(
    #[sol(rpc)]
    IMorpho,
    "data/abis/morpho.json"
);

#[tokio::main]
async fn main() {
    market::get_market().await;
    //market::read_events().await;
    vault::get_vault_data().await;
    vault::read_vaults().await;
}
