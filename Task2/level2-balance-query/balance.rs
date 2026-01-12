use anyhow::{anyhow, Context, Result};
use ethers::prelude::*;
use ethers::utils::format_ether;
use std::{env, str::FromStr};

async fn query_balance(address: Address, rpc_url: &str) -> Result<(U256, String)> {
    let provider = Provider::<Http>::try_from(rpc_url)
        .with_context(|| format!("RPC URL 无法解析或初始化 Provider: {rpc_url}"))?;

    let wei = provider
        .get_balance(address, None)
        .await
        .context("RPC 调用失败：get_balance")?;

    Ok((wei, format_ether(wei)))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args().skip(1);

    let address_str = args
        .next()
        .ok_or_else(|| anyhow!("缺少参数：地址。\n用法: cargo run -- <ADDRESS> [RPC_URL]"))?;

    let rpc_url = args
        .next()
        .unwrap_or_else(|| "https://sepolia-rollup.arbitrum.io/rpc".to_string());

    let address =
        Address::from_str(&address_str).with_context(|| format!("地址格式不正确: {address_str}"))?;

    let (wei, eth) = query_balance(address, &rpc_url).await?;

    println!("Address: {}", address);
    println!("Balance: {wei} wei = {eth} ETH");

    Ok(())
}
