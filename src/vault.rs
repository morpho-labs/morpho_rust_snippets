use alloy::{
    eips::BlockNumberOrTag,
    primitives::address,
    providers::{Provider, ProviderBuilder},
    rpc::types::Filter,
    sol,
    sol_types::SolEvent,
    transports::http::reqwest::Url,
};
use eyre::Result;

sol!(
    #[sol(rpc)]
    IVault,
    "data/abis/morpho_vault.json"
);

sol!(
    #[sol(rpc)]
    IVaultFactory,
    "data/abis/morpho_vault_factory.json"
);

pub async fn retrieve_vault_details(rpc_url: Url) -> Result<()> {
    let provider = ProviderBuilder::new().on_http(rpc_url);

    // This is a valid morpho vault address on Ethereum (Steakhouse USDC)
    let vault_address = address!("BEEF01735c132Ada46AA9aA4c54623cAA92A64CB"); // Steakhouse USDC
    let vault = IVault::new(vault_address, provider.clone());

    let vault_name = vault.name().call().await?._0;
    let underlying_token = vault.asset().call().await?._0;
    let total_assets = vault.totalAssets().call().await?.assets;
    println!("Vault {} at address {:#20x} has underlying token {:#20x} and currently has {} assets under management", vault_name, vault_address, underlying_token, total_assets);
    Ok(())
}

pub async fn retrieve_vault_activity_details(rpc_url: Url) -> Result<()> {
    let provider = ProviderBuilder::new().on_http(rpc_url);

    // This is a valid morpho vault address on Ethereum (Steakhouse USDC)
    let vault_address = address!("BEEF01735c132Ada46AA9aA4c54623cAA92A64CB");

    // Vault activity on a specific block range
    let filter = Filter::new()
        .address(vault_address)
        .from_block(BlockNumberOrTag::Number(21_200_000))
        .events([
            IVault::Deposit::SIGNATURE_HASH,
            IVault::Withdraw::SIGNATURE_HASH,
            IVault::Transfer::SIGNATURE_HASH,
            IVault::UpdateLastTotalAssets::SIGNATURE_HASH,
        ]);

    let logs = provider.get_logs(&filter).await?;

    println!("Got {} logs", logs.len());
    for log in logs {
        match log.topic0() {
            Some(&IVault::Deposit::SIGNATURE_HASH) => {
                let IVault::Deposit {
                    sender: _sender,
                    owner,
                    assets,
                    shares,
                } = log.log_decode()?.inner.data;
                println!(
                    "User {:#20x} deposited {} assets for {} shares",
                    owner, assets, shares
                );
            }
            Some(&IVault::Withdraw::SIGNATURE_HASH) => {
                let IVault::Withdraw {
                    sender: _sender,
                    receiver: _receiver,
                    owner,
                    assets,
                    shares,
                } = log.log_decode()?.inner.data;
                println!(
                    "User {:#20x} withdrew {} assets for {} shares",
                    owner, assets, shares
                );
            }
            Some(&IVault::Transfer::SIGNATURE_HASH) => {
                let IVault::Transfer { from, to, value } = log.log_decode()?.inner.data;
                println!(
                    "User {:#20x} transfered {} shares to user {:#20x}",
                    from, to, value
                );
            }
            Some(&IVault::UpdateLastTotalAssets::SIGNATURE_HASH) => {
                let IVault::UpdateLastTotalAssets { updatedTotalAssets } =
                    log.log_decode()?.inner.data;
                println!("Vault updated its total assets to {}", updatedTotalAssets)
            }
            // Missing SubmitTimelock, SetTimelock, SetSkimRecipient, SetFee, SetFeeRecipient
            // SubmitGuardian, SetGuardian, SubmitCap, SetCap, SubmitMarketRemoval, SetCurator, SetIsAllocator,
            // RevokePendingTimelock, RevokePendingCap, RevokePendingGuardian, RevokePendingMarketRemoval,
            // SetSupplyQueue, SetWithdrawQueue, ReallocateSupply, ReallocateWithdraw, AccrueInterest, Skim
            _ => (),
        }
    }
    Ok(())
}

pub async fn retrieve_vaults(rpc_url: Url) -> Result<()> {
    let provider = ProviderBuilder::new().on_http(rpc_url);

    // This factory emits an event when a vault is deployed
    let vault_factory_address = address!("A9c3D3a366466Fa809d1Ae982Fb2c46E5fC41101");

    // Filter over factory since deployment
    let filter = Filter::new()
        .address(vault_factory_address)
        .from_block(BlockNumberOrTag::Number(18_925_584));

    let logs = provider.get_logs(&filter).await?;

    println!("Got {} logs", logs.len());
    for log in logs {
        match log.topic0() {
            Some(&IVaultFactory::CreateMetaMorpho::SIGNATURE_HASH) => {
                let IVaultFactory::CreateMetaMorpho {
                    metaMorpho,
                    caller,
                    initialOwner: _initial_owner,
                    initialTimelock: _initial_timelock,
                    asset,
                    name,
                    symbol: _symbol,
                    salt: _salt,
                } = log.log_decode()?.inner.data;
                println!(
                    "Morpho vault {} at address {:#20x} created by {:#20x}, for asset {:#20x} ",
                    name, metaMorpho, caller, asset
                );
            }
            _ => (),
        }
    }
    Ok(())
}
