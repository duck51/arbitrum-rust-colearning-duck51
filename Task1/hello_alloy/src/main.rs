use alloy::providers::{Provider, ProviderBuilder};
use alloy::primitives::Address;
use std::error::Error;
use alloy::sol;

// 使用 sol! 宏定义一个 Solidity 合约接口
sol! { 
   // 启用 rpc 功能，使其可以通过 provider 调用链上合约
   #[sol(rpc)] 
   contract HelloWeb3 { 
        // 定义合约中的 hello_web3 方法
        // pure：不读取链上状态
        // public：公开方法
        // 返回 string 类型
        function hello_web3() pure public returns(string memory); 
   } 
} 

// 使用 tokio 异步运行时作为程序入口
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    // Arbitrum Sepolia 测试网的 RPC 地址，并解析为 Url 类型
    let rpc_url = "https://arbitrum-sepolia-rpc.publicnode.com".parse()?;
 
    let provider = ProviderBuilder::new().connect_http(rpc_url); 
    
    let latest_block = provider.get_block_number().await?;
    
    // 打印最新区块号
    println!("Latest block number: {latest_block}");
    println!("Hello web3");
   
    // 目标合约地址（HelloWeb3 合约部署地址）
    let contract_address: Address = 
        "0x3f1f78ED98Cd180794f1346F5bD379D5Ec47DE90".parse()?;
    
    // 使用合约地址和 provider 创建合约实例
    let contract = HelloWeb3::new(contract_address, provider);

    let result = contract.hello_web3().call().await?;

    println!("合约返回: {}", result);

    Ok(())
}
