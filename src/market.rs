use crate::arithmetic::{w_mul_down, w_taylor_compounded};
use alloy::providers::WsConnect;
use alloy::transports::http::reqwest::Url;
use alloy::{
    eips::BlockNumberOrTag,
    primitives::{address, B256, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::Filter,
    sol,
    sol_types::SolEvent,
};
use eyre::Result;
use futures_util::stream::StreamExt;
use std::time::{SystemTime, UNIX_EPOCH};
use IIRM::{Market, MarketParams};

// Code gen
sol!(
    #[sol(rpc)]
    IMorpho,
    "data/abis/morpho.json"
);
sol!(
    #[sol(rpc)]
    IOracle,
    "data/abis/morpho_oracle.json"
);

sol!(
    #[sol(rpc)]
    IIRM,
    "data/abis/adaptive_curve_irm.json"
);

pub async fn retrieve_market_info(rpc_url: Url) -> Result<()> {
    let provider = ProviderBuilder::new().on_http(rpc_url);

    // The morpho contract address (it contains all positions)
    let morpho_address = address!("BBBBBbbBBb9cC5e90e3b3Af64bdAF62C37EEFFCb");
    let morpho = IMorpho::new(morpho_address, provider.clone());

    // Getting market information
    // You can change this market id
    let market_id: B256 =
        "0xb48bb53f0f2690c71e8813f2dc7ed6fca9ac4b0ace3faa37b4a8e5ece38fa1a2".parse()?; // USD0++/USDC (86%) with AdaptiveCurve
    let market_params = morpho.idToMarketParams(market_id).call().await?;
    let market_data = morpho.market(market_id).call().await?;

    println!(
        "Market with id {:#32x} was updated for the last time at timestamp {}",
        market_id, market_data.lastUpdate
    );
    println!(
        "Market Params:\n- Collateral asset: {:#20x} \n- Loan asset: {:#20x} \n- LLTV: {} \n- Oracle: {:#20x} \n- IRM: {:#20x}",
        market_params.collateralToken,
        market_params.loanToken,
        market_params.lltv,
        market_params.oracle,
        market_params.irm
    );

    // Note that some interest might be lacking
    println!(
        "Market Data:\n- Fee: {} \n- Total borrow assets: {} \n- Total borrow shares: {} \n- Total supply assets: {} \n- Total supply shares: {}",
        market_data.fee,
        market_data.totalBorrowAssets,
        market_data.totalBorrowShares,
        market_data.totalSupplyAssets,
        market_data.totalSupplyShares
    );

    // Getting the price of the oracle
    let oracle = IOracle::new(market_params.oracle, provider.clone());
    let price = oracle.price().call().await?._0;
    println!("Current price of market oracle is {}", price);

    // Getting current rate from the IRM
    // Note we assume the IRM of this market to be the AdaptiveCurveIRM
    let irm = IIRM::new(market_params.irm, provider.clone());
    let borrow_rate = irm
        .borrowRateView(
            MarketParams {
                collateralToken: market_params.collateralToken,
                loanToken: market_params.loanToken,
                lltv: market_params.lltv,
                oracle: market_params.oracle,
                irm: market_params.irm,
            },
            Market {
                totalSupplyAssets: market_data.totalSupplyAssets,
                totalSupplyShares: market_data.totalSupplyShares,
                totalBorrowAssets: market_data.totalBorrowAssets,
                totalBorrowShares: market_data.totalBorrowShares,
                fee: market_data.fee,
                lastUpdate: market_data.lastUpdate,
            },
        )
        .call()
        .await?
        ._0;
    println!(
        "Current average rate since last update for this market is {}",
        borrow_rate
    );

    // Compute updated market total with interest
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let elapsed = U256::from(current_time) - U256::from(market_data.lastUpdate);
    let interest = w_mul_down(
        U256::from(market_data.totalBorrowAssets),
        w_taylor_compounded(borrow_rate, elapsed),
    );

    println!(
        "Market has {} pending interest since last update at {}, now it's {} so total supply asset will be {} and total borrow asset will be {}",
         interest,
         market_data.lastUpdate,
        current_time,
         U256::from(market_data.totalSupplyAssets)+interest,
         U256::from(market_data.totalBorrowAssets)+interest
    );

    // Getting a user position on this market
    // You can change this user
    let user = address!("171c53d55B1BCb725F660677d9e8BAd7fD084282");
    let position = morpho.position(market_id, user).call().await?;

    println!(
        "User {:#20x} position on this market:\n- Collateral: {}\n- Borrow shares: {}\n- Supply shares: {}",
        user, position.collateral, position.borrowShares, position.supplyShares
    );
    Ok(())
}

pub async fn retrieve_events_with_logs(rpc_url: Url) -> Result<()> {
    let provider = ProviderBuilder::new().on_http(rpc_url);

    // The morpho contract address
    let morpho_address = address!("BBBBBbbBBb9cC5e90e3b3Af64bdAF62C37EEFFCb");

    // You can change the block range (the script fails if there is too much logs on a range)
    let filter = Filter::new()
        .address(morpho_address)
        .from_block(BlockNumberOrTag::Number(21_250_000))
        .to_block(BlockNumberOrTag::Number(21_260_000));

    let logs = provider.get_logs(&filter).await?;

    println!("Got {} logs", logs.len());
    for log in logs {
        match log.topic0() {
            Some(&IMorpho::CreateMarket::SIGNATURE_HASH) => {
                let IMorpho::CreateMarket { id, marketParams } = log.log_decode()?.inner.data;
                println!(
                    "Market with id {:#32x} was created with params: {:#20x}, {:#20x}, {}, {:#20x}, {:#20x}",
                    id, marketParams.collateralToken, marketParams.loanToken, marketParams.lltv, marketParams.oracle, marketParams.irm
                );
            }
            Some(&IMorpho::Supply::SIGNATURE_HASH) => {
                let IMorpho::Supply {
                    id,
                    caller: _caller,
                    onBehalf,
                    assets,
                    shares: _shares,
                } = log.log_decode()?.inner.data;
                println!(
                    "User {:#20x} supplied {} assets on market {:#32x}",
                    onBehalf, assets, id
                );
            }
            Some(&IMorpho::Withdraw::SIGNATURE_HASH) => {
                let IMorpho::Withdraw {
                    id,
                    caller: _caller,
                    onBehalf,
                    receiver: _receiver,
                    assets,
                    shares: _shares,
                } = log.log_decode()?.inner.data;
                println!(
                    "User {:#20x} withdrew {} assets on market {:#32x}",
                    onBehalf, assets, id
                );
            }
            Some(&IMorpho::Borrow::SIGNATURE_HASH) => {
                let IMorpho::Borrow {
                    id,
                    caller: _caller,
                    onBehalf,
                    receiver: _receiver,
                    assets,
                    shares: _shares,
                } = log.log_decode()?.inner.data;
                println!(
                    "User {:#20x} borrowed {} assets on market {:#32x}",
                    onBehalf, assets, id
                );
            }
            Some(&IMorpho::Repay::SIGNATURE_HASH) => {
                let IMorpho::Repay {
                    id,
                    caller: _caller,
                    onBehalf,
                    assets,
                    shares: _shares,
                } = log.log_decode()?.inner.data;
                println!(
                    "User {:#20x} repaid {} assets on market {:#32x}",
                    onBehalf, assets, id
                );
            }
            Some(&IMorpho::SupplyCollateral::SIGNATURE_HASH) => {
                let IMorpho::SupplyCollateral {
                    id,
                    caller: _caller,
                    onBehalf,
                    assets,
                } = log.log_decode()?.inner.data;
                println!(
                    "User {:#20x} supplied {} assets as collateral on market {:#32x}",
                    onBehalf, assets, id
                );
            }
            Some(&IMorpho::WithdrawCollateral::SIGNATURE_HASH) => {
                let IMorpho::WithdrawCollateral {
                    id,
                    caller: _caller,
                    onBehalf,
                    receiver: _receiver,
                    assets,
                } = log.log_decode()?.inner.data;
                println!(
                    "User {:#20x} withdrew {} assets as collateral on market {:#32x}",
                    onBehalf, assets, id
                );
            }
            Some(&IMorpho::Liquidate::SIGNATURE_HASH) => {
                let IMorpho::Liquidate {
                    id,
                    caller,
                    borrower,
                    repaidAssets: _repaid_assets,
                    repaidShares: _repaid_shares,
                    seizedAssets: _seized_assets,
                    badDebtAssets: _bad_debt_assets,
                    badDebtShares: _bad_debt_shares,
                } = log.log_decode()?.inner.data;
                println!(
                    "User {:#20x} was liquidated by user {:#20x} on market {:#32x}",
                    borrower, caller, id
                );
            }
            // Miss SetOwner, SetFee, SetFeeRecipient, EnableIrm, EnableLltv, FlashLoan, SetAuthorization, IncrementNonce, AccrueInterest events
            _ => (),
        }
    }
    Ok(())
}

pub async fn retrieve_markets(rpc_url: Url) -> Result<()> {
    let provider = ProviderBuilder::new().on_http(rpc_url);

    // The morpho contract address
    let morpho_address = address!("BBBBBbbBBb9cC5e90e3b3Af64bdAF62C37EEFFCb");

    let filter = Filter::new()
        .address(morpho_address)
        .from_block(BlockNumberOrTag::Number(18_883_124))
        .event_signature(IMorpho::CreateMarket::SIGNATURE_HASH);

    let logs = provider.get_logs(&filter).await?;
    for log in logs {
        match log.topic0() {
            Some(&IMorpho::CreateMarket::SIGNATURE_HASH) => {
                let IMorpho::CreateMarket { id, marketParams } = log.log_decode()?.inner.data;
                println!(
                    "Market with id {:#32x} was created with params: {:#20x}, {:#20x}, {}, {:#20x}, {:#20x}",
                    id, marketParams.collateralToken, marketParams.loanToken, marketParams.lltv, marketParams.oracle, marketParams.irm
                );
            }
            _ => (),
        }
    }
    Ok(())
}

pub async fn subscribe_to_event_logs(rpc_url: &str) -> Result<()> {
    //let ws = WsConnect::new(rpc_url);
    let provider = ProviderBuilder::new()
        .on_ws(WsConnect::new(rpc_url))
        .await?;

    // The morpho contract address
    let morpho_address = address!("BBBBBbbBBb9cC5e90e3b3Af64bdAF62C37EEFFCb");

    let filter = Filter::new()
        .address(morpho_address)
        .from_block(BlockNumberOrTag::Latest);

    let sub = provider.subscribe_logs(&filter).await?;
    let mut stream = sub.into_stream();

    while let Some(log) = stream.next().await {
        match log.topic0() {
            Some(&IMorpho::CreateMarket::SIGNATURE_HASH) => {
                let IMorpho::CreateMarket { id, marketParams } = log.log_decode()?.inner.data;
                println!(
                    "Market with id {:#32x} was created with params: {:#20x}, {:#20x}, {}, {:#20x}, {:#20x}",
                    id, marketParams.collateralToken, marketParams.loanToken, marketParams.lltv, marketParams.oracle, marketParams.irm
                );
            }
            Some(&IMorpho::Supply::SIGNATURE_HASH) => {
                let IMorpho::Supply {
                    id,
                    caller: _caller,
                    onBehalf,
                    assets,
                    shares: _shares,
                } = log.log_decode()?.inner.data;
                println!(
                    "User {:#20x} supplied {} assets on market {:#32x}",
                    onBehalf, assets, id
                );
            }
            Some(&IMorpho::Withdraw::SIGNATURE_HASH) => {
                let IMorpho::Withdraw {
                    id,
                    caller: _caller,
                    onBehalf,
                    receiver: _receiver,
                    assets,
                    shares: _shares,
                } = log.log_decode()?.inner.data;
                println!(
                    "User {:#20x} withdrew {} assets on market {:#32x}",
                    onBehalf, assets, id
                );
            }
            Some(&IMorpho::Borrow::SIGNATURE_HASH) => {
                let IMorpho::Borrow {
                    id,
                    caller: _caller,
                    onBehalf,
                    receiver: _receiver,
                    assets,
                    shares: _shares,
                } = log.log_decode()?.inner.data;
                println!(
                    "User {:#20x} borrowed {} assets on market {:#32x}",
                    onBehalf, assets, id
                );
            }
            Some(&IMorpho::Repay::SIGNATURE_HASH) => {
                let IMorpho::Repay {
                    id,
                    caller: _caller,
                    onBehalf,
                    assets,
                    shares: _shares,
                } = log.log_decode()?.inner.data;
                println!(
                    "User {:#20x} repaid {} assets on market {:#32x}",
                    onBehalf, assets, id
                );
            }
            Some(&IMorpho::SupplyCollateral::SIGNATURE_HASH) => {
                let IMorpho::SupplyCollateral {
                    id,
                    caller: _caller,
                    onBehalf,
                    assets,
                } = log.log_decode()?.inner.data;
                println!(
                    "User {:#20x} supplied {} assets as collateral on market {:#32x}",
                    onBehalf, assets, id
                );
            }
            Some(&IMorpho::WithdrawCollateral::SIGNATURE_HASH) => {
                let IMorpho::WithdrawCollateral {
                    id,
                    caller: _caller,
                    onBehalf,
                    receiver: _receiver,
                    assets,
                } = log.log_decode()?.inner.data;
                println!(
                    "User {:#20x} withdrew {} assets as collateral on market {:#32x}",
                    onBehalf, assets, id
                );
            }
            Some(&IMorpho::Liquidate::SIGNATURE_HASH) => {
                let IMorpho::Liquidate {
                    id,
                    caller,
                    borrower,
                    repaidAssets: _repaid_assets,
                    repaidShares: _repaid_shares,
                    seizedAssets: _seized_assets,
                    badDebtAssets: _bad_debt_assets,
                    badDebtShares: _bad_debt_shares,
                } = log.log_decode()?.inner.data;
                println!(
                    "User {:#20x} was liquidated by user {:#20x} on market {:#32x}",
                    borrower, caller, id
                );
            }
            // Miss SetOwner, SetFee, SetFeeRecipient, EnableIrm, EnableLltv, FlashLoan, SetAuthorization, IncrementNonce, AccrueInterest events
            _ => (),
        }
    }
    Ok(())
}
