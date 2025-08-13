# Solana-EVM 跨链桥接实现

## 概述

本项目实现了一个简化的 Solana-EVM 双向跨链桥接系统。该系统基于一个核心假设：**跨链双方无条件相信对方会成功执行交易**。这意味着发起链在接收到跨链请求后，不再追踪后续状态。

## 核心设计原则

### 1. 两区块完成模型
跨链交易通过时间顺序的两个区块完成：

- **区块 N（请求区块）**：接收并记录跨链请求
- **区块 N+1（执行区块）**：执行跨链转账

### 2. 无状态追踪
- 发起链不维护跨链请求的执行状态
- 执行结果由共识层不再维护
- 如果执行失败，由发起人后续自行处理

## 实现架构

```
┌─────────────────┐         ┌─────────────────┐         ┌─────────────────┐
│   EVM Chain     │         │  Consensus      │         │  Solana Chain   │
│                 │         │     Layer       │         │                 │
├─────────────────┤         ├─────────────────┤         ├─────────────────┤
│                 │         │                 │         │                 │
│  Block N:       │         │  Monitor &      │         │  Block N:       │
│  - User sends   │────────>│  Query Events   │<────────│  - User sends   │
│    cross-chain  │         │                 │         │    cross-chain  │
│    request      │         │                 │         │    request      │
│                 │         │                 │         │                 │
│  Block N+1:     │<────────│  Inject Txs     │────────>│  Block N+1:     │
│  - Execute      │         │  into Block     │         │  - Execute      │
│    transfers    │         │                 │         │    transfers    │
│                 │         │                 │         │                 │
└─────────────────┘         └─────────────────┘         └─────────────────┘
```

## 两个关键实现及其技术方案

### 1. 跨链事件查询器 (`examples/bridge-consensus-monitor/`)

**实现目标**：获取区块中的跨链请求

**技术实现方式**：
- **事件日志查询**：使用 `eth_getLogs` RPC 方法查询特定区块的事件
- **事件解码**：通过 `alloy-sol-types` 自动解码 Solidity 事件
- **过滤机制**：
  ```rust
  // 构建事件过滤器
  let filter = Filter::new()
      .address(BRIDGE_ADDRESS)           // 指定桥接合约地址
      .topic0(EVENT_SIGNATURE_HASH)      // 指定事件签名
      .from_block(block_number)           // 指定区块范围
      .to_block(block_number);
  
  // 查询并解码事件
  let logs = provider.get_logs(&filter).await?;
  let event = BridgeRequestCreated::decode_log(&raw_log)?;
  ```

**核心代码逻辑**：
```rust
// main.rs 中的关键实现
async fn query_cross_chain_requests(&self, block_number: u64) -> Result<Vec<CrossChainRequest>> {
    // 1. 构建事件签名哈希
    let event_signature = BridgeRequestCreated::SIGNATURE_HASH;
    
    // 2. 创建过滤器，指定区块和合约
    let filter = Filter::new()
        .address(BRIDGE_ADDRESS)
        .topic0(event_signature)
        .from_block(block_number)
        .to_block(block_number);
    
    // 3. 通过 RPC 查询日志
    let logs = self.provider.get_logs(&filter).await?;
    
    // 4. 解析每个日志为跨链请求
    for log in logs {
        let event = BridgeRequestCreated::decode_log(&log)?;
        // 提取 requestId, sender, amount 等信息
    }
}
```

**输出结果**：
- 请求 ID、发送者地址、目标地址
- 跨链金额和手续费
- 区块号和交易哈希

### 2. 提款机制实现 (`examples/new-block-producer/`)

**实现目标**：在新区块中增加账户余额（处理 Solana → EVM 跨链）

**技术实现方式**：
- **共识层提款（Withdrawal）**：使用以太坊的原生提款机制
- **Engine API**：通过 `engine_forkchoiceUpdated` 和 `engine_getPayload` 控制区块生产
- **PayloadAttributes**：在载荷属性中包含提款信息

**核心代码逻辑**：
```rust
// 1. 创建提款对象（Withdrawal）
let withdrawal = Withdrawal {
    index: 0,                      // 提款索引
    validator_index: 0,            // 验证者索引
    address: target_address,       // 接收地址
    amount: 1_000_000_000,        // 金额：1 ETH = 1,000,000,000 Gwei
};

// 2. 构造包含提款的 PayloadAttributes
let payload_attributes = PayloadAttributes {
    timestamp,
    prev_randao: B256::ZERO,
    suggested_fee_recipient: Address::ZERO,
    withdrawals: Some(vec![withdrawal]),  // 👈 关键：包含提款
    parent_beacon_block_root: Some(B256::ZERO),
};

// 3. 通过 Engine API 请求构建载荷
let forkchoice_result = make_rpc_call(
    &client,
    "engine_forkchoiceUpdatedV3",
    json!([forkchoice_state, payload_attributes])  // 包含提款的载荷属性
).await?;

// 4. 获取构建的载荷
let payload = make_rpc_call(
    &client,
    "engine_getPayloadV3",
    json!([payload_id])
).await?;

// 5. 验证并提交载荷
let new_payload_result = make_rpc_call(
    &client,
    "engine_newPayloadV3",
    json!([execution_payload])
).await?;

// 6. 设置新区块为链头
let final_result = make_rpc_call(
    &client,
    "engine_forkchoiceUpdatedV3",
    json!([new_forkchoice_state, null])
).await?;
```

**关键技术点**：

1. **提款机制原理**：
   - 提款是以太坊共识层的原生功能
   - 每个提款包含验证者索引、地址和金额（Gwei）
   - 提款在区块执行时自动增加目标地址余额
   - 无需 gas 费用，由共识层直接处理

2. **Engine API 流程**：
   ```
   构造提款 → forkchoiceUpdated（请求构建）→ getPayload（获取载荷）
   → newPayload（验证）→ forkchoiceUpdated（出块）
   ```

3. **金额单位转换**：
   - 提款金额单位是 **Gwei**（1 ETH = 10^9 Gwei）
   - 不是 Wei（1 ETH = 10^18 Wei）
   - 例如：1 ETH = 1,000,000,000 Gwei

4. **验证提款生效**：
   ```rust
   // 查询余额变化
   let balance_before = eth_getBalance(address);
   // ... 执行提款 ...
   let balance_after = eth_getBalance(address);
   let change = balance_after - balance_before;
   // 应该增加 1 ETH
   ```

## 实现效果总结

### 效果 1：查询跨链请求
**实现方式**：通过 RPC 的 `eth_getLogs` 方法配合事件解码
- 无需维护链上状态
- 可查询任意历史区块
- 事件数据完整可靠

### 效果 2：增加账户余额
**实现方式**：通过共识层 Withdrawal（提款）机制
- 使用以太坊原生的提款功能
- 通过 Engine API 控制区块生产
- 零 gas 费用，共识层直接处理
- 金额单位为 Gwei（非 Wei）

## 跨链流程

### EVM → Solana

1. **用户发起**：
   ```bash
   cast send \
     --value 0.1ether \
     0x0000000000000000000000000000000000001000 \
     "bridgeToSolana(bytes32)" \
     0x0101010101010101010101010101010101010101010101010101010101010101
   ```

2. **区块 N**：
   - 桥接合约接收请求
   - 发出 `BridgeRequestCreated` 事件
   - 扣除 0.3% 手续费

3. **共识层处理**：
   - 使用 `bridge-consensus-monitor` 查询区块 N 的事件
   - 提取跨链请求信息
   - 准备 Solana 端执行

4. **区块 N+1（Solana）**：
   - 执行转账到目标地址
   - 金额 = 原始金额 - 手续费

### Solana → EVM

1. **用户发起**：
   - 在 Solana 端发起跨链请求

2. **区块 N（Solana）**：
   - 记录跨链请求

3. **共识层处理**：
   - 查询 Solana 区块的跨链请求
   - 通知 EVM 端的 `new-block-producer`

4. **区块 N+1（EVM）**：
   - `new-block-producer` 通过提款机制增加余额
   - 在 PayloadAttributes 中包含 Withdrawal 对象
   - 共识层自动处理，无需 gas 费用

## 测试步骤

### 1. 启动 Reth 节点
```bash
./target/release/reth node \
  --chain dev \
  --http \
  --http.api all
```

### 2. 部署桥接合约
```bash
./setup_bridge.sh
```

### 3. 发送测试跨链交易
```bash
# EVM → Solana
cast send \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --value 0.1ether \
  0x0000000000000000000000000000000000001000 \
  "bridgeToSolana(bytes32)" \
  0x0101010101010101010101010101010101010101010101010101010101010101
```

### 4. 查询跨链事件
```bash
cd examples/bridge-consensus-monitor
cargo run
```

### 5. 运行区块生产者（处理 Solana → EVM）
```bash
cd examples/new-block-producer
cargo run
```

## 关键特性

1. **无状态设计**：合约不存储任何跨链请求状态，仅通过事件记录
2. **自动手续费**：0.3% 手续费自动计算和扣除
3. **系统交易**：Solana → EVM 通过系统交易实现，无需 gas
4. **事件驱动**：所有跨链信息通过事件查询获取
5. **简化信任模型**：双方无条件信任执行，不追踪状态

## 注意事项

1. **执行保证**：系统假设跨链执行总是成功，不提供失败处理机制
2. **手续费**：固定 0.3% 手续费，不可配置
3. **区块依赖**：跨链完成依赖于连续的两个区块
4. **共识层责任**：共识层负责查询和传递跨链信息，但不维护执行状态

## 文件结构

```
.
├── bridge-contracts/                    # 桥接合约
│   └── src/
│       └── SolanaEVMBridge.sol         # EVM 端桥接合约
├── examples/
│   ├── bridge-consensus-monitor/       # 共识层监控器
│   │   └── src/
│   │       └── main.rs                # 查询跨链事件（eth_getLogs）
│   └── new-block-producer/            # 区块生产者
│       └── src/
│           └── main.rs                # 提款机制实现（Withdrawal via Engine API）
├── crates/
│   └── ethereum/
│       └── evm/
│           └── src/
│               └── lib.rs             # EVM 执行器（已移除 withdrawal_executor）
└── setup_bridge.sh                    # 合约部署脚本
```

## 更新日志

### 最新更新
- ✅ 移除了所有状态存储机制
- ✅ 简化为纯事件驱动架构
- ✅ 移除了 `withdrawal_executor` 模块
- ✅ 实现了共识层事件查询器（通过 eth_getLogs）
- ✅ 完成了提款机制实现（通过 Withdrawal 和 Engine API）

## 联系方式

如有问题，请联系相应负责人：
- EVM 侧实现：[EVM 负责人]
- Solana 侧实现：[Solana 负责人]
- 共识层集成：[共识层负责人]