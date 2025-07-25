# reth

[![bench status](https://github.com/paradigmxyz/reth/actions/workflows/bench.yml/badge.svg)](https://github.com/paradigmxyz/reth/actions/workflows/bench.yml)
[![CI status](https://github.com/paradigmxyz/reth/workflows/unit/badge.svg)][gh-ci]
[![cargo-lint status](https://github.com/paradigmxyz/reth/actions/workflows/lint.yml/badge.svg)][gh-lint]
[![Telegram Chat][tg-badge]][tg-url]

```
reth node --engine.accept-execution-requests-hash --datadir ./data \
  --chain dev \
  --authrpc.jwtsecret ./jwt.hex \
  --authrpc.addr 127.0.0.1 \
  --authrpc.port 8551 \
  --http \
  --ws \
  --rpc-max-connections 429496729 \
  --http.api txpool,trace,web3,eth,debug \
  --ws.api trace,web3,eth,debug
  ```

# Examples

This section provides easy-to-follow guides for key Reth examples. Each example demonstrates different aspects of interacting with Ethereum nodes.

## 🔧 Engine API Examples

### block-forkchoicev3 - Engine API Interaction
**What it does**: Demonstrates how to interact with Reth's Engine API using `engine_forkchoiceUpdatedV3` and `engine_getPayloadV3` methods.

**Use cases**:
- Building consensus clients
- Testing Engine API functionality  
- Understanding block production flow

**Quick Start**:
```bash
# 1. Start a Reth node with Engine API enabled
reth node --authrpc.port 8551 --authrpc.jwtsecret jwt.hex

# 2. Ensure you have jwt.hex file in project root
echo "0x$(openssl rand -hex 32)" > jwt.hex

# 3. Run the example
cargo run -p block-forkchoicev3
```

**What happens**:
1. Connects to Engine API using JWT authentication
2. Gets the latest block information
3. Constructs ForkchoiceState and PayloadAttributes
4. Calls `engine_forkchoiceUpdatedV3` to generate new payload
5. Uses the returned payloadId to fetch execution payload via `engine_getPayloadV3`

---

## 💰 Transaction Examples

### rpc-send-transaction - Complete Transaction Guide
**What it does**: Shows multiple ways to send transactions through RPC, from simple transfers to advanced transaction management.

**Use cases**:
- Learning transaction sending patterns
- Building wallet applications
- Automated transaction tools
- Testing transaction flows

**Quick Start**:
```bash
# Basic usage
cargo run -p rpc-send-transaction -- --rpc-url http://localhost:8545

# With custom parameters
cargo run -p rpc-send-transaction -- \
  --rpc-url http://localhost:8545 \
  --to 0x742d35cc6065c8532b5566d49e529c0e8bf1935b \
  --amount 0.5 \
  --trace

# Simple version
cargo run -p rpc-send-transaction --bin simple-client-simple
```

**Two approaches demonstrated**:
1. **Raw Transaction Method**: Manual signing and sending with `eth_sendRawTransaction`
2. **Alloy Provider Method**: Modern approach with automatic nonce/gas management

**Command line options**:
- `--rpc-url`: Node RPC endpoint (default: http://localhost:8545)
- `--to`: Recipient address
- `--amount`: ETH amount to send (default: 0.1)
- `--trace`: Enable detailed logging

⚠️ **Note**: Uses test private keys - never use these on mainnet!

---

## 📊 Monitoring Examples

### txpool-query - Transaction Pool Inspector  
**What it does**: Queries and displays all pending and queued transactions in the mempool with detailed information.

**Use cases**:
- Monitoring network congestion
- Analyzing gas price trends
- Building transaction tracking tools
- MEV research and analysis

**Quick Start**:
```bash
# Connect to local node
cargo run -p txpool-query

# View real-time transaction pool status
```

**Output includes**:
- **Pending transactions**: Ready to be included in next block
- **Queued transactions**: Waiting for prerequisites (nonce gaps, insufficient gas)
- **Detailed info**: Gas prices, values, sender/recipient addresses, transaction data

**Sample output**:
```json
{
  "from": "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266",
  "to": "0x70997970c51812dc3a010c7d01b50e0d17dc79c8",
  "value": "100000000000000000",
  "gas": "21000",
  "max_fee_per_gas": "1000000000",
  "nonce": "5",
  "input": ""
}
```

---

## 🚀 Getting Started

### Prerequisites
All examples require:
- Rust 1.70+ 
- A running Reth node (local or remote)

### Common Setup
```bash
# Clone the repository
git clone https://github.com/paradigmxyz/reth.git
cd reth

# Start a development node (for testing)
cargo run --bin reth -- node --dev --http --authrpc.port 8551

# In another terminal, run any example
cargo run -p <example-name>
```

### 🔄 重新配置开发节点 (Reconfiguring Development Node)

如果你修改了创世块配置文件 `crates/chainspec/res/genesis/dev.json`，需要重新初始化区块链数据：

#### 当你修改了 dev.json 配置后：

**步骤 1：停止当前节点**
```bash
# 如果节点正在运行，按 Ctrl+C 停止，或者：
pkill -f "reth node"
```

**步骤 2：清除旧数据**
```bash
# 删除旧的区块链数据（重要：创世块变更后必须清除）
rm -rf ./data
```

**步骤 3：重新启动节点**
```bash
# 使用你的命令重新启动（会自动重新初始化）
reth node \
  --datadir ./data \
  --chain dev \
  --authrpc.jwtsecret ./jwt.hex \
  --authrpc.addr 127.0.0.1 \
  --authrpc.port 8551 \
  --http \
  --ws \
  --rpc-max-connections 429496729 \
  --http.api txpool,trace,web3,eth,debug \
  --ws.api trace,web3,eth,debug
```

#### ⚠️ 重要说明：

- **为什么要删除数据目录？** 修改 `dev.json` 会改变创世块的哈希，旧的区块链数据与新配置不兼容
- **JWT 文件：** 确保 `jwt.hex` 文件存在，如果没有请创建：
  ```bash
  echo "0x$(openssl rand -hex 32)" > jwt.hex
  ```
- **配置变更：** 常见的 `dev.json` 修改包括：
  - 修改 `chainId`
  - 调整预分配账户余额 (`alloc` 字段)
  - 更改硬分叉激活时间
  - 修改初始gas限制

#### 🔍 验证节点重启成功：
```bash
# 检查节点是否正常运行
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
  http://localhost:8545

# 应该返回你在 dev.json 中设置的 chainId（如 1337）
```

### Authentication Notes
- **Engine API examples** require JWT authentication
- **RPC examples** work with standard HTTP/WS endpoints
- Development nodes typically don't require authentication for RPC

### Troubleshooting
- **Connection refused**: Ensure your node is running and accessible
- **JWT errors**: Check jwt.hex file exists and contains valid hex string
- **Transaction failures**: Verify account has sufficient balance
- **Port conflicts**: Adjust port numbers if defaults are occupied

---

## 📚 Example Categories

| Example | Category | Difficulty | Key Learning |
|---------|----------|------------|--------------|
| `txpool-query` | Monitoring | Beginner | RPC querying, mempool analysis |
| `rpc-send-transaction` | Transactions | Intermediate | Transaction creation, signing, sending |  
| `block-forkchoicev3` | Engine API | Advanced | Consensus integration, payload building |
| `block-producer` | Engine API | Expert | Complete block production workflow |

Each example is self-contained and includes detailed comments explaining the concepts and APIs used.

---

# 示例文档 (中文)

本节提供了Reth关键示例的简单易懂指南。每个示例演示了与以太坊节点交互的不同方面。

## 🔧 Engine API 示例

### block-producer - 真实区块生产流程
**功能说明**：演示完整的区块生产流程，模拟真实共识客户端与执行客户端的4步交互。

**使用场景**：
- 理解真实的区块生产机制
- 学习共识客户端与执行客户端交互
- 掌握完整的Engine API工作流程
- 区块奖励分配机制学习

**快速开始**：
```bash
# 1. 生成JWT密钥
openssl rand -hex 32 > jwt.hex

# 2. 启动支持Prague的Reth节点
reth node --engine.accept-execution-requests-hash --authrpc.jwtsecret jwt.hex

# 3. 运行示例
cargo run -p block-producer
```

**完整4步流程**：
1. **engine_forkchoiceUpdated** + PayloadAttributes → 请求构建载荷
2. **engine_getPayload** → 获取构建的载荷
3. **engine_newPayload** → 验证载荷
4. **engine_forkchoiceUpdated** (无PayloadAttributes) → 实际出块

**区块奖励配置**：
- `suggested_fee_recipient: Address::ZERO` → 奖励被销毁
- 修改为你的地址即可获得所有交易手续费和区块奖励

---

### block-forkchoicev3 - Engine API交互
**功能说明**：演示如何使用 `engine_forkchoiceUpdatedV3` 和 `engine_getPayloadV3` 方法与Reth的Engine API进行交互。

**使用场景**：
- 构建共识客户端
- 测试Engine API功能  
- 理解区块生产流程

**快速开始**：
```bash
# 1. 启动带有Engine API的Reth节点
reth node --authrpc.port 8551 --authrpc.jwtsecret jwt.hex

# 2. 确保项目根目录有jwt.hex文件
echo "0x$(openssl rand -hex 32)" > jwt.hex

# 3. 运行示例
cargo run -p block-forkchoicev3
```

**执行流程**：
1. 使用JWT认证连接到Engine API
2. 获取最新区块信息
3. 构造ForkchoiceState和PayloadAttributes
4. 调用 `engine_forkchoiceUpdatedV3` 生成新的payload
5. 使用返回的payloadId通过 `engine_getPayloadV3` 获取执行负载

---

## 💰 交易示例

### rpc-send-transaction - 完整交易指南
**功能说明**：展示通过RPC发送交易的多种方式，从简单转账到高级交易管理。

**使用场景**：
- 学习交易发送模式
- 构建钱包应用
- 自动化交易工具
- 测试交易流程

**快速开始**：
```bash
# 基本用法
cargo run -p rpc-send-transaction -- --rpc-url http://localhost:8545

# 自定义参数
cargo run -p rpc-send-transaction -- \
  --rpc-url http://localhost:8545 \
  --to 0x742d35cc6065c8532b5566d49e529c0e8bf1935b \
  --amount 0.5 \
  --trace

# 简化版本
cargo run -p rpc-send-transaction --bin simple-client-simple
```

**演示的两种方法**：
1. **原始交易方法**：使用 `eth_sendRawTransaction` 手动签名和发送
2. **Alloy Provider方法**：现代化方法，自动管理nonce和gas费用

**命令行选项**：
- `--rpc-url`：节点RPC端点 (默认: http://localhost:8545)
- `--to`：接收者地址
- `--amount`：发送的ETH数量 (默认: 0.1)
- `--trace`：启用详细日志

⚠️ **注意**：使用测试私钥 - 切勿在主网使用！

---

## 📊 监控示例

### txpool-query - 交易池检查器  
**功能说明**：查询并显示内存池中所有待处理和排队交易的详细信息。

**使用场景**：
- 监控网络拥堵情况
- 分析gas价格趋势
- 构建交易跟踪工具
- MEV研究和分析

**快速开始**：
```bash
# 连接到本地节点
cargo run -p txpool-query

# 查看实时交易池状态
```

**输出包含**：
- **待处理交易**：准备在下个区块中包含的交易
- **排队交易**：等待前置条件的交易（nonce间隙、gas不足等）
- **详细信息**：gas价格、金额、发送者/接收者地址、交易数据

**输出示例**：
```json
{
  "from": "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266",
  "to": "0x70997970c51812dc3a010c7d01b50e0d17dc79c8",
  "value": "100000000000000000",
  "gas": "21000",
  "max_fee_per_gas": "1000000000",
  "nonce": "5",
  "input": ""
}
```

---

## 🚀 入门指南

### 前置要求
所有示例都需要：
- Rust 1.70+ 
- 运行中的Reth节点（本地或远程）

### 通用设置
```bash
# 克隆仓库
git clone https://github.com/paradigmxyz/reth.git
cd reth

# 启动开发节点（用于测试）
cargo run --bin reth -- node --dev --http --authrpc.port 8551

# 在另一个终端运行任意示例
cargo run -p <示例名称>
```

### 认证说明
- **Engine API示例** 需要JWT认证
- **RPC示例** 适用于标准HTTP/WS端点
- 开发节点通常不需要RPC认证

### 故障排除
- **连接被拒绝**：确保节点正在运行且可访问
- **JWT错误**：检查jwt.hex文件存在且包含有效的十六进制字符串
- **交易失败**：验证账户有足够余额
- **端口冲突**：如果默认端口被占用，请调整端口号

---

## 📚 示例分类

| 示例 | 分类 | 难度 | 主要学习内容 |
|------|------|------|-------------|
| `txpool-query` | 监控 | 初级 | RPC查询、内存池分析 |
| `rpc-send-transaction` | 交易 | 中级 | 交易创建、签名、发送 |  
| `block-forkchoicev3` | Engine API | 高级 | 共识集成、负载构建 |
| `block-producer` | Engine API | 专家级 | 完整区块生产工作流程 |

每个示例都是独立的，包含详细注释解释所使用的概念和API。

---

# 代码详解 (Code Deep Dive)

## 🏭 block-producer 代码分析

### 整体架构和流程
`block-producer` 示例演示了**真实的区块生产流程**，完整模拟共识客户端与执行客户端的交互：

```rust
// 完整的4步区块生产流程：
// 1. engine_forkchoiceUpdated + PayloadAttributes → 请求构建载荷
// 2. engine_getPayload → 获取构建的载荷  
// 3. engine_newPayload → 验证载荷
// 4. engine_forkchoiceUpdated (无PayloadAttributes) → 实际出块
```

### JWT认证升级版
```rust
#[derive(serde::Serialize)]
struct Claims {
    iat: u64, // issued at
    exp: u64, // expires at - 新增过期时间
}

fn create_jwt_token(secret: &str) -> Result<String> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let claims = Claims {
        iat: now,
        exp: now + 3600, // 1小时后过期 - 更安全的做法
    };
    
    let key = EncodingKey::from_secret(hex::decode(secret)?.as_ref());
    let token = encode(&JwtHeader::new(Algorithm::HS256), &claims, &key)?;
    Ok(token)
}
```

### 通用RPC调用封装
```rust
async fn make_rpc_call(
    client: &Client,
    jwt: &str,
    method: &str,
    params: Value,
) -> Result<Value> {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params
    });

    let response = client
        .post("http://127.0.0.1:8551")
        .bearer_auth(jwt)
        .json(&request)
        .send()
        .await?;

    let response_json: Value = response.json().await?;
    
    // 统一错误处理
    if let Some(error) = response_json.get("error") {
        return Err(eyre::eyre!("RPC error: {}", error));
    }
    
    Ok(response_json["result"].clone())
}
```

### 区块奖励配置（关键特性）
```rust
// 🎯 这是决定谁获得区块奖励的关键配置
let fee_recipient = Address::ZERO; // 默认销毁奖励

// 实际使用时，替换为你的地址：
// let fee_recipient: Address = "0x742d35Cc8F3fE5e7e3F44A3e4D4e8e2e9d9F0C8A".parse()?;

let payload_attributes = PayloadAttributes {
    timestamp,
    prev_randao: B256::ZERO,
    suggested_fee_recipient: fee_recipient, // 👈 区块奖励接收者
    withdrawals: Some(vec![]),
    parent_beacon_block_root: Some(B256::ZERO), // Post-Cancun必需
};
```

### 步骤1：请求载荷构建
```rust
// 调用 engine_forkchoiceUpdated 请求构建新载荷
let forkchoice_result = make_rpc_call(
    &client, 
    &jwt, 
    "engine_forkchoiceUpdatedV3", 
    json!([forkchoice_state, payload_attributes])
).await?;

// 提取 payloadId 用于后续步骤
let payload_id = forkchoice_result.get("payloadId")
    .and_then(|id| id.as_str())
    .ok_or_else(|| eyre::eyre!("未收到 payloadId，无法继续"))?;
```

### 步骤2：获取构建的载荷
```rust
// 使用 payloadId 获取实际构建的执行载荷
let get_payload_result = make_rpc_call(
    &client, 
    &jwt, 
    "engine_getPayloadV3", 
    json!([payload_id])
).await?;

let execution_payload = get_payload_result.get("executionPayload")
    .ok_or_else(|| eyre::eyre!("响应中缺少 executionPayload"))?;

// 解析新区块信息
let new_block_number = u64::from_str_radix(
    execution_payload["blockNumber"].as_str().unwrap_or("0x0").trim_start_matches("0x"), 
    16
)?;
```

### 步骤3：智能API版本检测
```rust
// 自动检测是否需要使用 Prague 硬分叉的 V4 API
let is_prague = latest_block.get("requestsHash").is_some();

let (method, params) = if is_prague {
    println!("检测到 Prague 硬分叉，使用 engine_newPayloadV4");
    let requests_hash = "0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    ("engine_newPayloadV4", json!([execution_payload, [], "0x0000000000000000000000000000000000000000000000000000000000000000", requests_hash]))
} else {
    println!("使用 engine_newPayloadV3");
    ("engine_newPayloadV3", json!([execution_payload, [], "0x0000000000000000000000000000000000000000000000000000000000000000"]))
};

let new_payload_result = make_rpc_call(&client, &jwt, method, params).await?;
```

### 步骤4：实际出块（关键步骤）
```rust
// 🔄 这是真正"出块"的步骤 - 将新区块设置为链头
let new_block_hash_b256: B256 = new_block_hash.parse()?;

let final_forkchoice_state = ForkchoiceState {
    head_block_hash: new_block_hash_b256,      // 新区块作为头部
    safe_block_hash: new_block_hash_b256,      // 设置为安全区块  
    finalized_block_hash: parent_hash_b256,    // 父区块作为最终确认区块
};

// 调用 forkchoiceUpdated 但不带 payload attributes（只更新链头）
let final_forkchoice_result = make_rpc_call(
    &client, 
    &jwt, 
    "engine_forkchoiceUpdatedV3", 
    json!([final_forkchoice_state, serde_json::Value::Null]) // 👈 null = 不构建新载荷
).await?;
```

### 验证出块成功
```rust
// 验证区块确实被添加到链上
let updated_latest_block = make_rpc_call(&client, &jwt, "eth_getBlockByNumber", json!(["latest", false])).await?;

let updated_number = u64::from_str_radix(
    updated_latest_block["number"].as_str().unwrap_or("0x0").trim_start_matches("0x"), 
    16
)?;

if updated_number > current_number {
    println!("🎉 成功出块！");
    println!("   原区块: #{} -> 新区块: #{}", current_number, updated_number);
} else {
    println!("⚠️ 区块可能尚未更新到链上");
}
```

---

## 🔧 block-forkchoicev3 代码分析

### 核心结构体和导入
```rust
use alloy_rpc_types_engine::{ForkchoiceState, PayloadAttributes};
use alloy_primitives::{Address, B256};
use jsonwebtoken::{encode, Header, EncodingKey, Algorithm};

#[derive(serde::Serialize)]
struct Claims {
    iat: u64, // issued at timestamp
}
```

### JWT认证实现
```rust
fn create_jwt_token(secret: &str) -> eyre::Result<String> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let claims = Claims { iat: now };
    
    // 将十六进制字符串转换为字节
    let secret_bytes = hex::decode(secret)?;
    
    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(&secret_bytes),
    )?;
    
    Ok(token)
}
```
**关键点**：
- Engine API要求HS256算法的JWT认证
- `iat` (issued at) 时间戳防止重放攻击
- JWT密钥必须是32字节的十六进制字符串

### 获取最新区块信息
```rust
let get_block_request = serde_json::json!({
    "jsonrpc": "2.0",
    "id": 1,
    "method": "eth_getBlockByNumber",
    "params": ["latest", false]  // false = 不包含完整交易数据
});

let block_response = client
    .post(engine_url)
    .bearer_auth(&jwt_token)  // JWT认证
    .json(&get_block_request)
    .send()
    .await?;
```

### ForkchoiceState构造
```rust
let forkchoice_state = ForkchoiceState {
    head_block_hash: head_hash,      // 当前头部区块
    safe_block_hash: safe_hash,      // 安全区块（通常是父区块）
    finalized_block_hash: head_hash, // 最终确定区块
};
```

### PayloadAttributes配置
```rust
let payload_attributes = PayloadAttributes {
    timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
    prev_randao: B256::from(random::<[u8; 32]>()),           // 随机数
    suggested_fee_recipient: Address::ZERO,                  // 费用接收者
    withdrawals: Some(vec![]),                               // 取款列表
    parent_beacon_block_root: Some(B256::ZERO),            // 信标链区块根
};
```

---

## 💰 rpc-send-transaction 代码分析

### 方法一：原始交易发送

#### 私钥和地址管理
```rust
// 测试私钥（切勿在生产环境使用）
let private_key = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
let wallet = private_key.parse::<PrivateKeySigner>()?;
let from_address = wallet.address();
```

#### 获取交易参数
```rust
// 获取当前nonce
let nonce: u64 = provider.get_transaction_count(from_address).await?;

// 获取gas价格
let gas_price = provider.get_gas_price().await?;

// 估算gas限制
let gas_limit = provider.estimate_gas(&tx_request).await?;
```

#### 构造和签名交易
```rust
let tx = TransactionRequest::default()
    .with_from(from_address)
    .with_to(to_address)
    .with_value(U256::from(amount_wei))
    .with_gas_limit(gas_limit)
    .with_gas_price(gas_price)
    .with_nonce(nonce);

// 签名交易
let signature = wallet.sign_transaction(&tx).await?;
let signed_tx = tx.build(signature);

// 编码为原始字节
let raw_tx = signed_tx.encoded_2718();

// 发送原始交易
let tx_hash = provider.send_raw_transaction(&raw_tx).await?;
```

### 方法二：Alloy Provider高级功能

#### EIP-1559交易构造
```rust
let tx = TransactionRequest::default()
    .with_from(from_address)
    .with_to(to_address)
    .with_value(U256::from(amount_wei))
    .with_gas_limit(21000)
    .with_max_fee_per_gas(max_fee_per_gas)           // EIP-1559最大费用
    .with_max_priority_fee_per_gas(max_priority_fee); // 优先费用

// 使用Provider发送（自动管理nonce）
let pending_tx = provider.send_transaction(tx).await?;
let tx_hash = *pending_tx.tx_hash();
```

#### 交易确认等待
```rust
// 等待交易确认
match pending_tx.get_receipt().await {
    Ok(receipt) => {
        println!("交易确认，Gas使用: {}", receipt.gas_used);
        if receipt.status() {
            println!("交易成功执行");
        } else {
            println!("交易执行失败");
        }
    }
    Err(e) => println!("获取交易回执失败: {}", e),
}
```

### 错误处理模式
```rust
// 全面的错误处理
match provider.send_transaction(tx).await {
    Ok(pending) => {
        println!("交易已发送: {:?}", pending.tx_hash());
    }
    Err(RpcError::ErrorResp(err)) if err.code == -32000 => {
        println!("交易被拒绝: {}", err.message);
    }
    Err(e) => {
        println!("发送失败: {}", e);
    }
}
```

---

## 📊 txpool-query 代码分析

### 连接和查询设置
```rust
use alloy_provider::{Provider, ProviderBuilder};
use alloy_rpc_types_txpool::TxpoolContent;

// 创建Provider连接
let provider = ProviderBuilder::new()
    .connect_http("http://localhost:8545".parse()?);

// 直接调用txpool_content RPC方法
let txpool_content: TxpoolContent = provider
    .client()
    .request("txpool_content", ())
    .await?;
```

### 交易信息结构体
```rust
#[derive(Debug, Serialize)]
struct TransactionInfo {
    from: String,                           // 发送者地址
    to: Option<String>,                     // 接收者地址（可能为空，合约创建）
    value: String,                          // 转账金额
    gas: String,                           // Gas限制
    gas_price: Option<String>,             // Legacy gas价格
    max_fee_per_gas: Option<String>,       // EIP-1559最大费用
    max_priority_fee_per_gas: Option<String>, // EIP-1559优先费用
    nonce: String,                         // 交易序号
    input: String,                         // 交易数据（十六进制）
}
```

### 待处理交易处理
```rust
// 遍历所有待处理交易
for (address, txs) in txpool_content.pending {
    for (nonce, tx) in txs {
        let tx_info = TransactionInfo {
            from: format!("{:?}", address),
            to: tx.inner.to().map(|t| format!("{:?}", t)),
            value: format!("{:?}", tx.inner.value()),
            gas: format!("{:?}", tx.inner.gas_limit()),
            gas_price: tx.inner.gas_price().map(|p| format!("{:?}", p)),
            max_fee_per_gas: Some(format!("{:?}", tx.inner.max_fee_per_gas())),
            max_priority_fee_per_gas: Some(format!("{:?}", tx.inner.max_priority_fee_per_gas())),
            nonce: format!("{:?}", nonce),
            input: hex::encode(tx.inner.input()),
        };
        
        // 格式化输出JSON
        println!("{}", serde_json::to_string_pretty(&tx_info)?);
    }
}
```

### 排队交易分析
```rust
// 排队交易通常有以下特征：
// 1. Nonce不连续（存在gap）
// 2. Gas价格过低
// 3. 账户余额不足
// 4. 依赖其他交易先执行

for (address, txs) in txpool_content.queued {
    // 处理逻辑与pending相同，但这些交易暂时无法执行
    for (nonce, tx) in txs {
        // ... 相同的处理逻辑
    }
}
```

---

## 🚀 通用代码模式和最佳实践

### 1. 错误处理模式
```rust
// 使用eyre进行错误传播
async fn main() -> eyre::Result<()> {
    // 在任何可能失败的操作后使用?
    let result = risky_operation().await?;
    Ok(())
}

// 自定义错误处理
match operation_result {
    Ok(value) => process_success(value),
    Err(e) => {
        eprintln!("操作失败: {}", e);
        // 根据错误类型决定是否重试
        handle_specific_error(&e);
    }
}
```

### 2. 异步编程模式
```rust
#[tokio::main]  // 异步运行时
async fn main() -> eyre::Result<()> {
    // 并发执行多个异步操作
    let (result1, result2) = tokio::try_join!(
        async_operation_1(),
        async_operation_2()
    )?;
    
    Ok(())
}
```

### 3. JSON-RPC调用模式
```rust
// 标准JSON-RPC请求格式
let request = serde_json::json!({
    "jsonrpc": "2.0",      // 协议版本
    "id": 1,               // 请求ID，用于匹配响应
    "method": "method_name", // RPC方法名
    "params": [param1, param2] // 参数数组
});

// 发送请求并处理响应
let response = client.post(url)
    .json(&request)
    .send()
    .await?;

let json_response: serde_json::Value = response.json().await?;
```

### 4. 类型安全的地址和哈希处理
```rust
use alloy_primitives::{Address, B256, U256};

// 地址解析
let address = "0x742d35cc6065c8532b5566d49e529c0e8bf1935b"
    .parse::<Address>()?;

// 哈希解析
let hash = "0x1234...".parse::<B256>()?;

// 数值处理
let amount = U256::from(1000000000000000000u64); // 1 ETH in wei
```

### 5. 配置和环境管理
```rust
// 从环境变量或配置文件读取设置
let rpc_url = std::env::var("RPC_URL")
    .unwrap_or_else(|_| "http://localhost:8545".to_string());

// JWT密钥文件处理
let jwt_path = if std::path::Path::new("jwt.hex").exists() {
    "jwt.hex"
} else if std::path::Path::new("../../jwt.hex").exists() {
    "../../jwt.hex"
} else {
    return Err(eyre::eyre!("JWT文件未找到"));
};
```

每个示例都展示了以太坊开发中的关键概念，从基础的RPC调用到高级的Engine API交互，为开发者提供了完整的学习路径。

<!-- Links -->
[gh-ci]: https://github.com/paradigmxyz/reth/actions/workflows/unit.yml
[gh-lint]: https://github.com/paradigmxyz/reth/actions/workflows/lint.yml
[tg-badge]: https://img.shields.io/badge/telegram-@reth-blue?logo=telegram&logoColor=white
[tg-url]: https://t.me/paradigm_reth