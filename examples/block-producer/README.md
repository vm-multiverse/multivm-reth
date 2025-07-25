# Block Producer Example

这个示例演示真实的 Engine API 区块生产流程，展示共识客户端和执行客户端之间的交互方式。

## 🔧 节点配置要求

这个示例需要根据你的 reth 节点配置选择正确的 Engine API 版本：

### Prague 配置 (推荐)
如果你的节点配置了 Prague 硬分叉，你需要：

```bash
# 启动支持 Prague 的 reth 节点
reth node --engine.accept-execution-requests-hash

# 运行示例 (使用 engine_newPayloadV4)
cargo run -p block-producer
```

### Cancun 配置
如果你想演示 `engine_newPayloadV3`，请使用 Cancun 配置：

```bash
# 启动仅支持 Cancun 的 reth 节点 (禁用 Prague)
reth node --dev --engine.legacy

# 运行示例 (使用 engine_newPayloadV3)
cargo run -p block-producer
```

## 🚀 使用说明

1. **准备 JWT 文件**
   ```bash
   # 生成 JWT 密钥
   openssl rand -hex 32 > jwt.hex
   ```

2. **启动 reth 节点**
   ```bash
   # Prague 配置 (推荐)
   reth node --engine.accept-execution-requests-hash --authrpc.jwtsecret jwt.hex
   
   # 或 Cancun 配置 (用于演示 V3)
   reth node --dev --engine.legacy --authrpc.jwtsecret jwt.hex
   ```

3. **运行示例**
   ```bash
   cargo run -p block-producer
   ```

## 📋 功能特性

- ✅ **真实的 Engine API 流程**: 展示正确的 forkchoiceUpdated → getPayload → newPayload → forkchoiceUpdated 完整顺序
- ✅ **自动检测硬分叉配置**: 代码会自动检测节点的硬分叉配置
- ✅ **智能 API 版本选择**: 根据节点配置使用 V3 或 V4
- ✅ **教育价值**: 展示共识客户端和执行客户端的实际交互方式
- ✅ **完整的错误处理**: 提供清晰的错误信息和调试输出

## 🔍 输出示例

```
🚀 真实的区块生产示例
这个示例演示了共识客户端如何与执行客户端交互来生产区块

✅ 找到 JWT: ../../jwt.hex
📊 获取最新区块信息...
当前区块: #0, 哈希: 0x2f980576...

🔧 构造载荷属性:
  - 时间戳: 1752599457
  - 建议的手续费接收者: 0x0000000000000000000000000000000000000000

📤 步骤 1: 调用 engine_forkchoiceUpdated 请求构建载荷...
✅ ForkchoiceUpdated 响应: { "payloadId": "0xa3fd219422d9085b", ... }
🎯 获得 payloadId: 0xa3fd219422d9085b

📦 步骤 2: 调用 engine_getPayload 获取构建的载荷...
🎉 成功获取载荷！新区块号: #1

🔍 步骤 3: 调用 engine_newPayload 验证载荷...
✅ NewPayload 响应: { "status": "VALID", ... }
🎉 载荷验证成功！

🔄 步骤 4: 调用 engine_forkchoiceUpdated 实际出块...
🎯 将新区块设置为链头: 0x80e65283b...
✅ 最终 ForkchoiceUpdated 响应: { "payloadStatus": { "status": "VALID" }, ... }

🔍 验证新区块是否成功出块...
🎉 成功出块！
   原区块: #0 -> 新区块: #1

📋 完整流程总结:
1. ✅ 通过 engine_forkchoiceUpdated 请求构建载荷
2. ✅ 通过 engine_getPayload 获取构建的载荷
3. ✅ 通过 engine_newPayload 验证载荷
4. ✅ 通过 engine_forkchoiceUpdated 实际出块

这就是真实环境中共识客户端和执行客户端的交互方式！
```

## ❗ 常见问题

### "missing requests hash" 错误
```
解决方案：你的节点配置了 Prague 硬分叉，需要启动时添加：
--engine.accept-execution-requests-hash
```

### "requests hash cannot be accepted" 错误
```
解决方案：重新启动 reth 时添加标志：
reth node --engine.accept-execution-requests-hash
```

### 想演示 engine_newPayloadV3
```
解决方案：使用 Cancun 配置：
reth node --dev --engine.legacy
```

## 💰 区块奖励和出块权

### 谁获得区块奖励？

区块奖励由 `PayloadAttributes` 中的 `suggested_fee_recipient` 字段决定：

```rust
let fee_recipient = Address::ZERO; // 当前设置：奖励被销毁
// 要获得奖励，改为你的地址：
// let fee_recipient: Address = "0x你的以太坊地址".parse()?;
```

### 区块奖励包含什么？

1. **交易手续费**：区块中所有交易的 gas fee
2. **优先费用**：用户支付的小费（tip）
3. **MEV 收入**：最大可提取价值（如果使用 MEV-Boost）

### 在开发环境中测试奖励

要在开发环境中看到真实的奖励效果：

1. **修改 fee_recipient**：
   ```rust
   let fee_recipient: Address = "0x你的地址".parse()?;
   ```

2. **发送一些交易**：
   ```bash
   # 在另一个终端发送交易来产生手续费
   cast send --rpc-url http://localhost:8545 \
     --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
     0x742d35Cc8F3fE5e7e3F44A3e4D4e8e2e9d9F0C8A \
     --value 1ether
   ```

3. **运行区块生产**：
   ```bash
   cargo run -p block-producer
   ```

4. **检查奖励**：
   ```bash
   cast balance 0x你的地址 --rpc-url http://localhost:8545
   ```

### 真实网络中的出块权

在主网中：
- **验证者**通过质押 32 ETH 获得出块权
- **共识客户端**决定轮到哪个验证者出块
- **执行客户端**按照共识客户端的指示构建区块
- **MEV-Boost**：验证者可以将构建权委托给专业构建者

### 开发环境 vs 真实网络

| 环境 | 出块权 | 奖励归属 | 限制 |
|------|--------|----------|------|
| 开发环境 | 任何人都可以调用 Engine API | `fee_recipient` 指定的地址 | 仅限测试 |
| 真实网络 | 质押验证者 | 验证者的 `fee_recipient` | 需要 32 ETH 质押 |

## 🎯 学习目标

通过这个示例，你将学会：
- 理解完整的区块生产流程：forkchoiceUpdated → getPayload → newPayload → forkchoiceUpdated
- 如何使用 engine_forkchoiceUpdated 请求载荷构建
- 如何使用 engine_getPayload 获取构建的载荷
- 如何使用 engine_newPayload 验证载荷
- 如何使用 engine_forkchoiceUpdated 实际出块（将区块添加到链上）
- 理解共识客户端和执行客户端的完整交互方式
- 如何处理不同的硬分叉配置和 API 版本

## 📚 相关资源

- [Engine API 规范](https://github.com/ethereum/execution-apis/tree/main/src/engine)
- [Reth 引擎 API 文档](https://reth.rs/)
- [Prague 硬分叉特性](https://github.com/ethereum/execution-apis/blob/main/src/engine/prague.md) 