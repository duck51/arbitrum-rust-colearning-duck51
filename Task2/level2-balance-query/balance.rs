use anyhow::{anyhow, Context, Result};
use ethers::prelude::*;
use ethers::utils::format_ether;
use std::env;
use std::str::FromStr;

/// 查询指定地址在 Arbitrum Sepolia 测试网的 ETH 余额（wei），并返回 (wei, formatted_eth)
async fn query_arb_sepolia_eth_balance(address: Address, rpc_url: &str) -> Result<(U256, String)> {
    // Provider 用于连接 EVM JSON-RPC :contentReference[oaicite:1]{index=1}
    let provider = Provider::<Http>::try_from(rpc_url)
        .with_context(|| format!("RPC URL 无法解析或初始化 Provider: {rpc_url}"))?;

    // get_balance 返回 wei（U256）
    let wei: U256 = provider
        .get_balance(address, None)
        .await
        .context("RPC 调用失败：get_balance")?;

    // 转成可读 ETH 字符串（从 wei -> ETH）
    // ethers_core/utils 中提供 format_ether / format_units :contentReference[oaicite:2]{index=2}
    let eth_readable = format_ether(wei);

    Ok((wei, eth_readable))
}

#[tokio::main]
async fn main() -> Result<()> {
    // 用法：
    // cargo run --manifest-path level2-balance-query/Cargo.toml -- <ADDRESS> [RPC_URL]
    let mut args = env::args().skip(1);

    let address_str = args
        .next()
        .ok_or_else(|| anyhow!("缺少参数：地址。\n用法: balance <ADDRESS> [RPC_URL]"))?;

    // 默认用 Arbitrum 文档列出的 Arbitrum Sepolia 公共 RPC :contentReference[oaicite:3]{index=3}
    let default_rpc = "https://sepolia-rollup.arbitrum.io/rpc";
    let rpc_url = args.next().unwrap_or_else(|| default_rpc.to_string());

    let address =
        Address::from_str(&address_str).with_context(|| format!("地址格式不正确: {address_str}"))?;

    let (wei, eth) = query_arb_sepolia_eth_balance(address, &rpc_url).await?;

    println!("Address : {:?}", address);
    println!("Balance : {} ETH", eth);
    println!("Wei     : {}", wei);
    println!("Balance: {wei} wei = {eth} ETH");


    Ok(())
}
