use dotenvy::dotenv;
use ethers::prelude::*;
use ethers::providers::Middleware;
use ethers::types::transaction::eip1559::Eip1559TransactionRequest;
use ethers::types::transaction::eip2718::TypedTransaction; // ✅ 修复：导入 TypedTransaction
use ethers::utils;
use std::cmp::max;
use std::env;
use std::sync::Arc;
use std::time::Duration;

const ARB_SEPOLIA_CHAIN_ID: u64 = 421_614;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //加载 .env（私钥从环境变量读，禁止硬编码）
    dotenv().ok();

    //读取私钥
    let private_key_raw = env::var("ARB_SEPOLIA_PRIVATE_KEY")
        .map_err(|_| "缺少环境变量 ARB_SEPOLIA_PRIVATE_KEY\n请确认 .env 文件存在且内容正确")?;
    let private_key = private_key_raw.trim();

    println!(
        "读取到的私钥 (前10字符): {}",
        &private_key[0..10.min(private_key.len())]
    );
    println!("私钥总长度: {}", private_key.len());

    let wallet: LocalWallet = private_key
        .parse::<LocalWallet>()
        .map_err(|e| format!("私钥解析失败: {}\n请检查是否真的是 64 位 hex 私钥（可带 0x）", e))?
        .with_chain_id(ARB_SEPOLIA_CHAIN_ID);

    //连接 RPC
    let rpc_url = "https://arbitrum-sepolia-rpc.publicnode.com";
    let provider = Provider::<Http>::try_from(rpc_url)?
        .interval(Duration::from_millis(250));

    println!("连接 RPC: {}", rpc_url);

    //校验 chain_id
    let chain_id = provider.get_chainid().await?.as_u64();
    println!("链ID: {} (预期 Arbitrum Sepolia = 421614)", chain_id);
    if chain_id != ARB_SEPOLIA_CHAIN_ID {
        return Err(format!(
            "RPC 链ID不匹配：拿到 {chain_id}，预期 {ARB_SEPOLIA_CHAIN_ID}"
        )
        .into());
    }

    //SignerMiddleware：负责签名 + 发送
    let client = Arc::new(SignerMiddleware::new(provider.clone(), wallet));

    let to_address_str = "0x0c4bF5740D5f34195AC5B02B89ab8a5e9C54d4F7";
    let to_address: Address = to_address_str
        .parse()
        .map_err(|_| format!("无效的接收地址: {}", to_address_str))?;

    let value = utils::parse_ether("0.00025")?;


    let from_address = client.address();
    println!("发送方 (A): {:?}", from_address);
    println!("接收方 (B): {:?}", to_address);

    //查询余额
    let balance = client.get_balance(from_address, None).await?;
    println!(
        "当前余额: {} wei ≈ {:.6} ETH",
        balance,
        balance.as_u128() as f64 / 1e18
    );

    //读取 baseFee + 建议 EIP-1559 费用
    let latest_block = provider
        .get_block(BlockNumber::Latest)
        .await?
        .ok_or("拿不到最新区块")?;
    let base_fee = latest_block
        .base_fee_per_gas
        .ok_or("最新区块没有 base_fee_per_gas（RPC 不支持？）")?;

    let (suggest_max_fee, suggest_tip) = provider.estimate_eip1559_fees(None).await?;

    let min_need = base_fee + suggest_tip;
    let final_max_fee = max(suggest_max_fee, min_need) * 12 / 10;

    println!("baseFee(最新区块): {} wei", base_fee);
    println!("建议 tip         : {} wei", suggest_tip);
    println!("建议 maxFee      : {} wei", suggest_max_fee);
    println!("最终 maxFee(+20%): {} wei", final_max_fee);

    //构造 EIP-1559 交易
    let mut tx1559 = Eip1559TransactionRequest {
        from: Some(from_address),
        to: Some(NameOrAddress::Address(to_address)),
        value: Some(value),
        max_fee_per_gas: Some(final_max_fee),
        max_priority_fee_per_gas: Some(suggest_tip),
        ..Default::default()
    };

    //估算 gas
    println!("估计 gas...");
    let typed_for_estimate: TypedTransaction = tx1559.clone().into();
    let gas_estimate = client.estimate_gas(&typed_for_estimate, None).await?;
    println!("估计 gas limit: {}", gas_estimate);


    let gas_limit = gas_estimate * 12 / 10;
    tx1559.gas = Some(gas_limit);
    println!("最终 gas limit(+20%): {}", gas_limit);

    //预估费用：value + gas_limit * max_fee
    let est_gas_cost = gas_limit * final_max_fee;
    let est_total = value + est_gas_cost;

    println!(
        "预估 gas 成本: {} wei ≈ {:.6} ETH",
        est_gas_cost,
        est_gas_cost.as_u128() as f64 / 1e18
    );
    println!(
        "预估总花费(value+gas): {} wei ≈ {:.6} ETH",
        est_total,
        est_total.as_u128() as f64 / 1e18
    );

    // 余额检查（更安全：包含 gas 上限）
    if balance < est_total {
        return Err(format!(
            "余额不足：当前 ≈{:.6} ETH，需要至少 ≈{:.6} ETH（含 gas 预估）",
            balance.as_u128() as f64 / 1e18,
            est_total.as_u128() as f64 / 1e18
        )
        .into());
    }

    // 12) 发送交易
    println!("正在发送交易...");
    let pending_tx = client.send_transaction(tx1559, None).await?;

    let tx_hash = *pending_tx;
    println!("\n交易已发送！");
    println!("交易哈希: {:?}", tx_hash);
    println!("查看: https://sepolia.arbiscan.io/tx/{}\n", tx_hash);

    // 13) 等待确认（可选）
    println!("等待确认...");
    match pending_tx.await {
        Ok(Some(receipt)) => {
            println!("交易成功确认！✅");
            println!("区块号: {:?}", receipt.block_number);
            println!("Gas 使用: {:?}", receipt.gas_used);
            println!("状态: {:?}", receipt.status);
        }
        Ok(None) => println!("交易发送但未在预期时间内确认（仍可能在 pending）"),
        Err(e) => println!("确认失败: {}", e),
    }

    Ok(())
}
