use alloy::primitives::Address;
use eyre::Result;
use reqwest;
use serde_json::{self, Value};
use std::{collections::HashMap, str::FromStr};

#[derive(Debug)]
pub struct Asset {
    token: Address,
    price: Option<f64>,
    decimals: u64,
    symbol: String,
    chain: u64,
}

pub async fn get_usd_prices() -> Result<Vec<Asset>> {
    let mut res: Vec<Asset> = Vec::new();

    let client = reqwest::Client::new();
    let mut body = HashMap::new();
    let query: &str = "query {
        assets(first: 1000) {
            items {
            priceUsd,
            address,
            decimals,
            symbol
            chain {
              id
            }
          }

        }
      }";
    let api_url: &str = "https://blue-api.morpho.org/graphql";

    body.insert("query", query);

    let data = client
        .post(api_url)
        .json(&body)
        .send()
        .await?
        .text()
        .await?;
    let raw_data: Value = serde_json::from_str(data.as_str())?;
    let prices = raw_data["data"]["assets"]["items"].clone();
    if prices.is_array() {
        for x in prices.as_array().unwrap().iter() {
            if x["address"].is_string() {
                let s = x["address"].to_string();
                let address = Address::from_str(&s[1..s.len() - 1])?;
                res.push(Asset {
                    token: address,
                    price: x["price"].as_f64(),
                    decimals: x["decimals"].as_u64().unwrap(),
                    symbol: String::from(x["symbol"].as_str().unwrap()),
                    chain: x["chain"]["id"].as_u64().unwrap(),
                })
            }
        }
    }
    Ok(res)
}
