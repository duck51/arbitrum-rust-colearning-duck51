use ethers::{
    abi::Abi,
    contract::Contract,
    middleware::Middleware,
    providers::{Http, Provider},
    types::{Address, U256},
};
use std::{str::FromStr, sync::Arc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 开始查询 Arbitrum Sepolia 测试网上的 USDC 合约");
    println!("{}", "=".repeat(50));
    
    // 1. 连接到 Arbitrum Sepolia 测试网
    let rpc_url = "https://sepolia-rollup.arbitrum.io/rpc";
    println!("📡 连接 RPC: {}", rpc_url);
    
    let provider = Provider::<Http>::try_from(rpc_url)?;
    
    // 测试连接
    let block_number = provider.get_block_number().await?;
    let chain_id = provider.get_chainid().await?;
    
    println!("✅ 网络连接成功");
    println!("   当前区块: {}", block_number);
    println!("   链 ID: {} (Arbitrum Sepolia)", chain_id);
    println!("{}", "=".repeat(50));
    
    // 2. 设置要查询的合约地址
    let contract_address = "0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d";
    println!("📋 合约地址: {}", contract_address);
    
    // 3. 将字符串地址转换为 Address 类型
    let address = match Address::from_str(contract_address) {
        Ok(addr) => {
            println!("✅ 地址格式正确");
            addr
        }
        Err(e) => {
            println!("❌ 地址格式错误: {}", e);
            return Err(e.into());
        }
    };
    
    // 4. 定义合约的 ABI（我们只需要查询函数）
    println!("📄 加载合约 ABI...");
    let abi_json = r#"[
        {
            "inputs": [],
            "name": "name",
            "outputs": [{"internalType": "string", "name": "", "type": "string"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "symbol",
            "outputs": [{"internalType": "string", "name": "", "type": "string"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "decimals",
            "outputs": [{"internalType": "uint8", "name": "", "type": "uint8"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "totalSupply",
            "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        }
    ]"#;
    
    // 5. 解析 ABI
    let abi: Abi = match serde_json::from_str(abi_json) {
        Ok(abi) => {
            println!("✅ ABI 解析成功");
            abi
        }
        Err(e) => {
            println!("❌ ABI 解析失败: {}", e);
            return Err(e.into());
        }
    };
    
    println!("{}", "=".repeat(50));
    println!("📊 开始查询合约信息");
    println!("{}", "=".repeat(50));
    
    // 6. 创建合约实例
    let client = Arc::new(provider);
    let contract = Contract::new(address, abi, client.clone());
    
    // 7. 查询合约信息
    let mut success_count = 0;
    let total_queries = 4;
    
    // 7.1 查询合约名称
    println!("1️⃣  查询 name()...");
    match contract.method::<_, String>("name", ()) {
        Ok(method) => {
            match method.call().await {
                Ok(name) => {
                    println!("   ✅ 合约名称: {}", name);
                    success_count += 1;
                }
                Err(e) => println!("   ❌ 查询失败: {}", e),
            }
        }
        Err(e) => println!("   ❌ 构建查询失败: {}", e),
    }
    
    // 7.2 查询代币符号
    println!("2️⃣  查询 symbol()...");
    match contract.method::<_, String>("symbol", ()) {
        Ok(method) => {
            match method.call().await {
                Ok(symbol) => {
                    println!("   ✅ 代币符号: {}", symbol);
                    success_count += 1;
                }
                Err(e) => println!("   ❌ 查询失败: {}", e),
            }
        }
        Err(e) => println!("   ❌ 构建查询失败: {}", e),
    }
    
    // 7.3 查询小数位数
    println!("3️⃣  查询 decimals()...");
    match contract.method::<_, u8>("decimals", ()) {
        Ok(method) => {
            match method.call().await {
                Ok(decimals) => {
                    println!("   ✅ 小数位数: {}", decimals);
                    success_count += 1;
                }
                Err(e) => println!("   ❌ 查询失败: {}", e),
            }
        }
        Err(e) => println!("   ❌ 构建查询失败: {}", e),
    }
    
    // 7.4 查询总供应量
    println!("4️⃣  查询 totalSupply()...");
    match contract.method::<_, U256>("totalSupply", ()) {
        Ok(method) => {
            match method.call().await {
                Ok(total_supply) => {
                    // USDC 有6位小数，所以除以 10^6
                    let total = total_supply.as_u128() as f64 / 1_000_000.0;
                    println!("   ✅ 总供应量: {:.2} USDC", total);
                    println!("     原始值: {} wei", total_supply);
                    success_count += 1;
                }
                Err(e) => println!("   ❌ 查询失败: {}", e),
            }
        }
        Err(e) => println!("   ❌ 构建查询失败: {}", e),
    }
    
    println!("{}", "=".repeat(50));
    println!("📈 查询结果统计");
    println!("   成功: {}/{}", success_count, total_queries);
    
    if success_count == total_queries {
        println!("🎉 所有查询都成功完成！");
    } else if success_count > 0 {
        println!("⚠️");
    } else {
        println!("❌");
    }
    
    println!("{}", "=".repeat(50));
    
    Ok(())
}