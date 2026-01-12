use anyhow::{Context, Result};
use ethers::prelude::*;
use ethers::utils::{format_ether, format_units};
use std::fmt;


#[derive(Debug)]
struct GasBill {
    price_wei_per_gas: U256, // wei / gas
    limit_gas: U256,         // gas
    cost_wei: U256,          // wei
    price_gwei_per_gas: String,
    cost_eth: String,
}
const BASE_TX_GAS: u64 = 21_000;

impl fmt::Display for GasBill {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "🧾  Arbitrum Transfer Gas Estimate")?;
        writeln!(f, "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")?;
        writeln!(f, "⛽  Gas Price   : {} wei/gas", self.price_wei_per_gas)?;
        writeln!(f, "✨  Gas Price   : {} gwei/gas", self.price_gwei_per_gas)?;
        writeln!(f, "🧱  Gas Limit   : {} gas", self.limit_gas)?;
        writeln!(f, "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")?;
        writeln!(f, "💰  Fee (wei)   : {}", self.cost_wei)?;
        writeln!(f, "🌈  Fee (ETH)   : ~{} ETH", self.cost_eth)?;
        writeln!(f, "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
    }
}

/// 1) 动态获取 gas price（provider.get_gas_price） ,gas_limit 使用 21,000,费用公式：fee_wei = gas_price_wei * gas_limit
async fn fetch_bill(rpc_addr: &str) -> Result<GasBill> {
    let net = Provider::<Http>::try_from(rpc_addr)
        .with_context(|| format!("RPC URL 无法解析或初始化 Provider: {rpc_addr}"))?;

    // 动态获取实时 gas price（非硬编码）
    let price_wei_per_gas = net
        .get_gas_price()
        .await
        .context("RPC 调用失败：get_gas_price")?;

    let limit_gas = U256::from(BASE_TX_GAS);

    // Gas Fee(wei) = Gas Price(wei/gas) × Gas Limit(gas)
    let cost_wei = price_wei_per_gas * limit_gas;

    // 输出友好：同时展示 gwei 与 ETH
    let price_gwei_per_gas =
        format_units(price_wei_per_gas, "gwei").context("format_units 失败：wei -> gwei")?;
    let cost_eth = format_ether(cost_wei);

    Ok(GasBill {
        price_wei_per_gas,
        limit_gas,
        cost_wei,
        price_gwei_per_gas,
        cost_eth,
    })
}

fn print_pretty(rpc_addr: &str, bill: &GasBill) {
    println!();
    println!("🔗 RPC Endpoint: {rpc_addr}");
    println!("{bill}");
}

#[tokio::main]
async fn main() -> Result<()> {
    let rpc_addr = "https://arbitrum-sepolia-rpc.publicnode.com";

    let bill = fetch_bill(rpc_addr).await?;
    print_pretty(rpc_addr, &bill);

    Ok(())
}
