# Morpho Rust Snippets

## Overview

This repository contains rust snippets to get on-chain data from the Morpho protocol.

## Prerequisites

Rust and Cargo.

## Files

- `market.rs`: Morpho market snippets
    - `retrieve_market_info`: retrieves various information (accounting, rates, user position) about a market
    - `retrieve_markets`: retrieves all market created on Morpho
    - `retrieve_events_with_logs`: retrieve logs on Morpho over a specific range of blocks
    - `subscribe_to_event_log`: listen to all events emitted on Morpho
- `vault.rs`: Morpho vault snippets
    - `retrive_vault_details`: retrieve various information (accounting) about a vault
    - `retrieve_vault_activity_details`: retrieve vault interactions (deposit, withdraw, transfer and interest accrual) on a specific Morpho vault
    - `retrieve_vaults`: retrieve all vaults created by the Morpho vault factory
- `arithmetic.rs`: Morpho protocol arithmetic logic (equivalent of [MathLib.sol](https://github.com/morpho-org/morpho-blue/blob/main/src/libraries/MathLib.sol))
- `api.rs`: Morpho API snippet

## How to use it

Run the main file with
```
cargo run
``
