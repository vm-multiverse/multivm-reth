# New Block Producer - 通过提款机制增加账户余额

这个示例演示了如何通过共识层的提款（Withdrawals）机制给指定地址增加 ETH，而不需要创建交易。

## 功能特点

- 🎯 给指定地址 `0xFE165617d15AD8B55F3821D6647E8E35E14A7f20` 增加 1 ETH
- 💸 使用提款机制（Withdrawals）而非交易
- 🔄 完整的区块生产流程
- 📊 实时验证余额变化

## 工作原理

### 提款机制说明

在以太坊中，提款（Withdrawals）是唯一不通过交易就能修改账户余额的标准方式：

1. **资金来源**：信标链（共识层）的验证者质押和奖励
2. **资金去向**：执行层的指定地址
3. **本质**：在执行层"铸造"等额的 ETH

### 执行流程

1. **获取当前状态**
   - 查询目标地址当前余额
   - 获取最新区块信息

2. **构建包含提款的载荷**
   ```rust
   let withdrawal = Withdrawal {
       index: 0,
       validator_index: 0,
       address: target_address,
       amount: 1_000_000_000, // 1 ETH = 10^9 Gwei
   };
   ```

3. **Engine API 调用序列**
   - `engine_forkchoiceUpdatedV3` - 请求构建包含提款的载荷
   - `engine_getPayloadV3` - 获取构建的载荷
   - `engine_newPayloadV3` - 验证载荷
   - `engine_forkchoiceUpdatedV3` - 实际出块

4. **验证结果**
   - 检查目标地址余额增加
   - 确认新区块包含提款

## 运行要求

### 1. 启动 Reth 节点

```bash
# 开发模式启动，启用 Engine API
reth node \
    --dev \
    --dev.block-time 12 \
    --http \
    --http.api all \
    --authrpc.addr 127.0.0.1 \
    --authrpc.port 8551 \
    --authrpc.jwtsecret jwt.hex
```

### 2. 准备 JWT 文件

确保项目根目录有 `jwt.hex` 文件：
```bash
# 如果没有，可以生成一个
openssl rand -hex 32 > jwt.hex
```

### 3. 运行示例

```bash
cargo run -p new-block-producer
```

## 预期输出

```
🚀 新区块生产示例：通过提款增加账户余额
💰 目标地址: 0xfe165617d15ad8b55f3821d6647e8e35e14a7f20
   将通过提款机制增加 1 ETH

📊 获取目标地址当前余额...
当前余额: 0 ETH (0 Wei)

💸 创建提款:
  - 接收地址: 0xfe165617d15ad8b55f3821d6647e8e35e14a7f20
  - 金额: 1 ETH (1,000,000,000 Gwei)

✅ 载荷包含 1 个提款
🎉 载荷验证成功！

💰 验证余额变化...
新余额: 1 ETH (1000000000000000000 Wei)
✅ 余额增加: 1 ETH

🎉 成功通过提款机制增加账户余额！
```

## 关键代码片段

### 创建提款
```rust
let withdrawal = Withdrawal {
    index: 0,                    // 提款索引
    validator_index: 0,          // 验证者索引
    address: target_address,     // 接收地址
    amount: 1_000_000_000,      // 金额（Gwei）
};
```

### 包含提款的载荷属性
```rust
let payload_attributes = PayloadAttributes {
    timestamp,
    prev_randao: B256::ZERO,
    suggested_fee_recipient: Address::ZERO,
    withdrawals: Some(vec![withdrawal]), // 👈 关键：包含提款
    parent_beacon_block_root: Some(B256::ZERO),
};
```

## 注意事项

1. **单位转换**
   - 提款金额单位是 **Gwei**（1 Gwei = 10^9 Wei）
   - 1 ETH = 10^9 Gwei = 10^18 Wei

2. **权限要求**
   - 需要访问 Engine API（端口 8551）
   - 需要正确的 JWT 认证

3. **状态更新**
   - 余额变化可能需要等待几秒才能查询到
   - 新区块需要被设置为链头才会生效

## 与标准 block-producer 的区别

| 特性 | block-producer | new-block-producer |
|-----|---------------|-------------------|
| 主要目的 | 演示区块生产流程 | 演示提款机制 |
| 余额修改 | 无 | 通过提款增加 1 ETH |
| PayloadAttributes | 空提款列表 | 包含 1 个提款 |
| 验证步骤 | 只验证出块 | 验证出块和余额变化 |

## 扩展应用

这个示例展示的提款机制可以用于：

1. **验证者奖励分发**：将质押奖励发送给验证者
2. **系统级余额调整**：在不创建交易的情况下修改余额
3. **协议升级**：在硬分叉时进行余额迁移或调整

## 相关文档

- [Engine API 规范](https://github.com/ethereum/execution-apis/tree/main/src/engine)
- [EIP-4895: Beacon chain push withdrawals](https://eips.ethereum.org/EIPS/eip-4895)
- [Reth Engine API 实现](../../crates/rpc/rpc-engine-api/)

## 故障排除

### 问题：找不到 jwt.hex
确保在项目根目录创建 JWT 文件：
```bash
openssl rand -hex 32 > jwt.hex
```

### 问题：连接被拒绝
确保 Reth 节点正在运行并启用了 Engine API：
```bash
# 检查端口是否监听
lsof -i :8551
```

### 问题：余额未变化
- 等待几秒让状态更新
- 检查新区块是否成功出块
- 查看节点日志确认提款是否被处理