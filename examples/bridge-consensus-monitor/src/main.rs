//! 桥接共识监控器
//!
//! 查询当前区块的跨链事件

use alloy_primitives::{Address, Bytes, B256, U256, LogData};
use alloy_sol_types::{sol, SolEvent};
use ethers::providers::{Http, Middleware, Provider};
use ethers::types::{BlockNumber, Filter, Log};
use eyre::Result;
use std::str::FromStr;

// 定义桥接合约地址
const BRIDGE_ADDRESS: &str = "0x0000000000000000000000000000000000001000";
const RPC_URL: &str = "http://localhost:8545";

// 使用 alloy-sol-types 定义事件结构
sol! {
    /// 桥接请求创建事件
    event BridgeRequestCreated(
        uint256 indexed requestId,
        address indexed sender,
        bytes32 indexed solanaRecipient,
        uint256 amount,
        uint256 fee,
        uint256 timestamp,
        uint256 blockNumber,
        uint256 nonce
    );
}

/// 跨链请求数据
#[derive(Debug, Clone)]
struct CrossChainRequest {
    request_id: U256,
    sender: Address,
    solana_recipient: B256,
    amount: U256,
    fee: U256,
    timestamp: U256,
    nonce: U256,
    tx_hash: B256,
}

/// 桥接共识监控器
struct BridgeConsensusMonitor {
    provider: Provider<Http>,
    bridge_address: Address,
}

impl BridgeConsensusMonitor {
    /// 创建新的监控器实例
    fn new(rpc_url: &str) -> Result<Self> {
        let provider = Provider::<Http>::try_from(rpc_url)?;
        let bridge_address = Address::from_str(BRIDGE_ADDRESS)?;
        
        Ok(Self {
            provider,
            bridge_address,
        })
    }

    /// 查询当前区块的跨链事件
    async fn query_current_block(&mut self) -> Result<()> {
        println!("========================================");
        println!("跨链事件查询");
        println!("========================================");
        println!("桥接合约: {}", self.bridge_address);
        println!("RPC URL: {}", RPC_URL);
        println!();

        // 获取当前区块号
        let current_block = self.provider.get_block_number().await?;
        let current_block_num = current_block.as_u64();
        println!("当前区块: #{}", current_block_num);
        println!("========================================");
        
        // 处理当前区块
        self.process_block(current_block_num).await?;
        
        // 显示统计信息
        println!("\n========================================");
        println!("查询完成");
        println!("========================================");
        
        Ok(())
    }

    /// 处理单个区块，查询其中的跨链事件
    async fn process_block(&self, block_number: u64) -> Result<()> {
        // 查询该区块的跨链事件
        let requests = self.query_cross_chain_requests(block_number).await?;
        
        if requests.is_empty() {
            println!("\n✗ 区块 #{} 没有跨链事件", block_number);
        } else {
            println!("\n✓ 区块 #{} 发现 {} 个跨链事件:", block_number, requests.len());
            println!();
            
            // 显示每个跨链请求
            for (index, request) in requests.iter().enumerate() {
                println!("【跨链事件 #{}】", index + 1);
                println!("----------------------------------------");
                self.display_request(request);
            }
            
            // 显示汇总信息
            let total_amount: U256 = requests.iter().map(|r| r.amount).fold(U256::ZERO, |a, b| a + b);
            let total_fees: U256 = requests.iter().map(|r| r.fee).fold(U256::ZERO, |a, b| a + b);
            
            println!("【汇总信息】");
            println!("----------------------------------------");
            println!("  总跨链金额: {} ETH", format_ether(total_amount));
            println!("  总手续费: {} ETH", format_ether(total_fees));
            println!("  事件数量: {}", requests.len());
        }
        
        Ok(())
    }

    /// 查询指定区块的跨链请求事件
    async fn query_cross_chain_requests(&self, block_number: u64) -> Result<Vec<CrossChainRequest>> {
        // 构建事件过滤器
        let event_signature = BridgeRequestCreated::SIGNATURE_HASH;
        
        let filter = Filter::new()
            .address(ethers::types::Address::from_slice(self.bridge_address.as_slice()))
            .topic0(ethers::types::H256::from_slice(event_signature.as_slice()))
            .from_block(BlockNumber::Number(block_number.into()))
            .to_block(BlockNumber::Number(block_number.into()));
        
        // 查询日志
        let logs = self.provider.get_logs(&filter).await?;
        
        // 解析日志为跨链请求
        let mut requests = Vec::new();
        for log in logs {
            if let Some(request) = self.parse_log_to_request(log)? {
                requests.push(request);
            }
        }
        
        Ok(requests)
    }

    /// 解析日志为跨链请求
    fn parse_log_to_request(&self, log: Log) -> Result<Option<CrossChainRequest>> {
        // 确保日志来自桥接合约
        if log.address != ethers::types::Address::from_slice(self.bridge_address.as_slice()) {
            return Ok(None);
        }
        
        // 解析事件数据
        let topics: Vec<B256> = log.topics.iter()
            .map(|t| B256::from_slice(t.as_bytes()))
            .collect();
            
        let log_data = LogData::new(
            topics,
            Bytes::from(log.data.to_vec()),
        ).ok_or_else(|| eyre::eyre!("Failed to create LogData"))?;
        
        let raw_log = alloy_primitives::Log {
            address: Address::from_slice(log.address.as_bytes()),
            data: log_data,
        };
        
        // 解码事件
        let decoded = BridgeRequestCreated::decode_log(&raw_log)?;
        let event = decoded.data;
        
        Ok(Some(CrossChainRequest {
            request_id: event.requestId,
            sender: event.sender,
            solana_recipient: event.solanaRecipient,
            amount: event.amount,
            fee: event.fee,
            timestamp: event.timestamp,
            nonce: event.nonce,
            tx_hash: B256::from_slice(log.transaction_hash.unwrap().as_bytes()),
        }))
    }

    /// 显示跨链请求详情
    fn display_request(&self, request: &CrossChainRequest) {
        println!("  请求ID: {}", request.request_id);
        println!("  发送者: {}", request.sender);
        println!("  Solana接收地址: 0x{}", hex::encode(request.solana_recipient));
        println!("  跨链金额: {} ETH", format_ether(request.amount));
        println!("  手续费: {} ETH", format_ether(request.fee));
        println!("  时间戳: {}", request.timestamp);
        println!("  Nonce: {}", request.nonce);
        println!("  交易哈希: 0x{}", hex::encode(request.tx_hash));
        println!();
    }

}

/// 格式化 wei 为 ETH
fn format_ether(wei: U256) -> String {
    let eth = wei.to_string().parse::<f64>().unwrap_or(0.0) / 1e18;
    format!("{:.6}", eth)
}

#[tokio::main]
async fn main() -> Result<()> {
    // 创建监控器
    let mut monitor = BridgeConsensusMonitor::new(RPC_URL)?;
    
    // 查询当前区块的跨链事件
    monitor.query_current_block().await?;
    
    Ok(())
}