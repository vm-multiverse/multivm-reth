//! 新区块生产示例：通过提款机制增加账户余额
//! 
//! 这个示例展示了如何通过共识层的提款机制给指定地址增加 ETH：
//! 1. 获取最新区块信息
//! 2. 调用 engine_forkchoiceUpdated 请求构建新载荷（包含提款）
//! 3. 调用 engine_getPayload 获取构建的载荷
//! 4. 调用 engine_newPayload 提交载荷进行验证
//! 5. 再次调用 engine_forkchoiceUpdated 实际出块
//!
//! 特别之处：在 PayloadAttributes 中包含提款信息，给指定地址增加 1 ETH
//!
//! 运行前请确保：
//! - reth 节点运行在 localhost:8551
//! - 项目根目录有 jwt.hex 文件
//!
//! 运行命令：cargo run -p new-block-producer

use alloy_rpc_types_engine::{ForkchoiceState, PayloadAttributes};
use alloy_eips::eip4895::Withdrawal;
use alloy_primitives::{Address, B256};
use eyre::Result;
use jsonwebtoken::{encode, Header as JwtHeader, EncodingKey, Algorithm};
use reqwest::Client;
use serde_json::{json, Value};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(serde::Serialize)]
struct Claims {
    iat: u64,
    exp: u64,
}

fn create_jwt_token(secret: &str) -> Result<String> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let claims = Claims {
        iat: now,
        exp: now + 3600, // 1小时过期
    };
    
    let key = EncodingKey::from_secret(hex::decode(secret)?.as_ref());
    let token = encode(&JwtHeader::new(Algorithm::HS256), &claims, &key)?;
    Ok(token)
}

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
    
    if let Some(error) = response_json.get("error") {
        return Err(eyre::eyre!("RPC error: {}", error));
    }
    
    Ok(response_json["result"].clone())
}

// HTTP RPC 调用（用于 eth_* 方法）
async fn make_http_rpc_call(
    client: &Client,
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
        .post("http://127.0.0.1:8545")  // HTTP RPC 端口
        .json(&request)
        .send()
        .await?;

    let response_json: Value = response.json().await?;
    
    if let Some(error) = response_json.get("error") {
        return Err(eyre::eyre!("RPC error: {}", error));
    }
    
    Ok(response_json["result"].clone())
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 新区块生产示例：通过提款增加账户余额");
    println!("这个示例演示如何通过共识层提款机制给指定地址增加 1 ETH\n");
    
    // 目标地址：将获得 1 ETH
    let target_address: Address = "0x128088d42dd2b6722C3AcAd432aF3264a9D7bDA4".parse()?;
    println!("💰 目标地址: {}", target_address);
    println!("   将通过提款机制增加 1 ETH\n");
    
    // 1. 读取 JWT
    let jwt_paths = ["jwt.hex", "./jwt.hex", "../jwt.hex", "../../jwt.hex"];
    let mut jwt_secret = String::new();
    
    for path in &jwt_paths {
        if Path::new(path).exists() {
            jwt_secret = std::fs::read_to_string(path)?.trim().to_string();
            println!("✅ 找到 JWT: {}", path);
            break;
        }
    }
    
    if jwt_secret.is_empty() {
        return Err(eyre::eyre!("未找到 jwt.hex 文件"));
    }
    
    let jwt = create_jwt_token(&jwt_secret)?;
    let client = Client::new();
    
    // 2. 获取目标地址当前余额（通过 HTTP RPC）
    println!("📊 获取目标地址当前余额...");
    let balance_before = match make_http_rpc_call(
        &client,
        "eth_getBalance",
        json!([format!("{:?}", target_address), "latest"])
    ).await {
        Ok(balance) => {
            let balance_wei = u128::from_str_radix(
                balance.as_str().unwrap_or("0x0").trim_start_matches("0x"),
                16
            )?;
            let balance_eth = balance_wei as f64 / 1e18;
            println!("当前余额: {} ETH ({} Wei)", balance_eth, balance_wei);
            Some(balance_wei)
        },
        Err(e) => {
            println!("⚠️  无法获取余额: {}", e);
            println!("   继续执行提款操作...");
            None
        }
    };
    
    // 3. 获取最新区块
    println!("\n📊 获取最新区块信息...");
    let latest_block = make_rpc_call(&client, &jwt, "eth_getBlockByNumber", json!(["latest", false])).await?;
    
    let current_number = u64::from_str_radix(
        latest_block["number"].as_str().unwrap_or("0x0").trim_start_matches("0x"), 
        16
    )?;
    let parent_hash = latest_block["hash"].as_str().unwrap_or("0x0");
    let parent_hash_b256: B256 = parent_hash.parse()?;
    
    println!("当前区块: #{}, 哈希: {}", current_number, parent_hash);
    
    // 4. 构造 ForkchoiceState
    let forkchoice_state = ForkchoiceState {
        head_block_hash: parent_hash_b256,
        safe_block_hash: parent_hash_b256,
        finalized_block_hash: parent_hash_b256,
    };
    
    // 5. 构造包含提款的 PayloadAttributes
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    
    // 🎯 关键：创建提款对象，给目标地址增加 1 ETH
    let withdrawal = Withdrawal {
        index: 0,
        validator_index: 0,
        address: target_address,
        amount: 1_000_000_000, // 1 ETH = 1,000,000,000 Gwei
    };
    
    println!("\n💸 创建提款:");
    println!("  - 接收地址: {}", target_address);
    println!("  - 金额: 1 ETH (1,000,000,000 Gwei)");
    println!("  - 验证者索引: 0");
    println!("  - 提款索引: 0");
    
    let payload_attributes = PayloadAttributes {
        timestamp,
        prev_randao: B256::ZERO,
        suggested_fee_recipient: Address::ZERO, // 手续费接收者
        withdrawals: Some(vec![withdrawal]), // 👈 包含提款
        parent_beacon_block_root: Some(B256::ZERO),
    };
    
    println!("\n🔧 构造载荷属性:");
    println!("  - 时间戳: {}", timestamp);
    println!("  - 包含 1 个提款");
    
    // 6. 调用 engine_forkchoiceUpdated 请求构建载荷
    println!("\n📤 步骤 1: 调用 engine_forkchoiceUpdated 请求构建载荷（包含提款）...");
    
    // 序列化 ForkchoiceState 和 PayloadAttributes
    let forkchoice_state_json = json!({
        "headBlockHash": format!("{:?}", forkchoice_state.head_block_hash),
        "safeBlockHash": format!("{:?}", forkchoice_state.safe_block_hash),
        "finalizedBlockHash": format!("{:?}", forkchoice_state.finalized_block_hash),
    });
    
    let payload_attributes_json = json!({
        "timestamp": format!("0x{:x}", payload_attributes.timestamp),
        "prevRandao": format!("{:?}", payload_attributes.prev_randao),
        "suggestedFeeRecipient": format!("{:?}", payload_attributes.suggested_fee_recipient),
        "withdrawals": payload_attributes.withdrawals.as_ref().map(|w| {
            w.iter().map(|withdrawal| json!({
                "index": format!("0x{:x}", withdrawal.index),
                "validatorIndex": format!("0x{:x}", withdrawal.validator_index),
                "address": format!("{:?}", withdrawal.address),
                "amount": format!("0x{:x}", withdrawal.amount),
            })).collect::<Vec<_>>()
        }),
        "parentBeaconBlockRoot": format!("{:?}", payload_attributes.parent_beacon_block_root.unwrap_or(B256::ZERO)),
    });
    
    let forkchoice_result = make_rpc_call(
        &client,
        &jwt,
        "engine_forkchoiceUpdatedV3",
        json!([forkchoice_state_json, payload_attributes_json])
    ).await?;
    
    println!("✅ ForkchoiceUpdated 响应: {}", serde_json::to_string_pretty(&forkchoice_result)?);
    
    // 检查是否有 payloadId
    let payload_id = forkchoice_result.get("payloadId")
        .and_then(|id| id.as_str())
        .ok_or_else(|| eyre::eyre!("未收到 payloadId，无法继续"))?;
    
    println!("🎯 获得 payloadId: {}", payload_id);
    
    // 7. 调用 engine_getPayload 获取构建的载荷
    println!("\n📦 步骤 2: 调用 engine_getPayload 获取构建的载荷...");
    
    let get_payload_result = make_rpc_call(
        &client, 
        &jwt, 
        "engine_getPayloadV3", 
        json!([payload_id])
    ).await?;
    
    let execution_payload = get_payload_result.get("executionPayload")
        .ok_or_else(|| eyre::eyre!("响应中缺少 executionPayload"))?;
    
    // 验证载荷中包含提款
    if let Some(withdrawals) = execution_payload.get("withdrawals") {
        if let Some(withdrawals_array) = withdrawals.as_array() {
            println!("✅ 载荷包含 {} 个提款", withdrawals_array.len());
            for (i, w) in withdrawals_array.iter().enumerate() {
                println!("   提款 {}: 地址 {}, 金额 {} Gwei", 
                    i, 
                    w.get("address").and_then(|a| a.as_str()).unwrap_or("unknown"),
                    w.get("amount").and_then(|a| a.as_str()).unwrap_or("0")
                );
            }
        }
    } else {
        println!("⚠️ 载荷中没有提款字段");
    }
    
    let new_block_number = u64::from_str_radix(
        execution_payload["blockNumber"].as_str().unwrap_or("0x0").trim_start_matches("0x"), 
        16
    )?;
    
    println!("\n🎉 成功获取载荷！新区块号: #{}", new_block_number);
    println!("   区块哈希: {}", execution_payload.get("blockHash").and_then(|h| h.as_str()).unwrap_or("unknown"));
    
    // 8. 调用 engine_newPayload 提交载荷进行验证
    println!("\n🔍 步骤 3: 调用 engine_newPayload 验证载荷...");
    
    // 检测是否需要 V4 (Prague)
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
    println!("✅ NewPayload 响应: {}", serde_json::to_string_pretty(&new_payload_result)?);
    
    let status = new_payload_result.get("status")
        .and_then(|s| s.as_str())
        .unwrap_or("UNKNOWN");
    
    if status != "VALID" {
        return Err(eyre::eyre!("载荷验证失败，状态: {}", status));
    }
    
    println!("🎉 载荷验证成功！");
    
    // 9. 调用 engine_forkchoiceUpdated 实际出块
    println!("\n🔄 步骤 4: 调用 engine_forkchoiceUpdated 实际出块...");
    
    let new_block_hash = execution_payload.get("blockHash")
        .and_then(|h| h.as_str())
        .ok_or_else(|| eyre::eyre!("无法获取新区块哈希"))?;
    
    let new_block_hash_b256: B256 = new_block_hash.parse()?;
    
    let final_forkchoice_state = ForkchoiceState {
        head_block_hash: new_block_hash_b256,
        safe_block_hash: new_block_hash_b256,
        finalized_block_hash: parent_hash_b256,
    };
    
    println!("🎯 将新区块设置为链头: {}", new_block_hash);
    
    let final_forkchoice_state_json = json!({
        "headBlockHash": format!("{:?}", final_forkchoice_state.head_block_hash),
        "safeBlockHash": format!("{:?}", final_forkchoice_state.safe_block_hash),
        "finalizedBlockHash": format!("{:?}", final_forkchoice_state.finalized_block_hash),
    });
    
    let final_forkchoice_result = make_rpc_call(
        &client,
        &jwt,
        "engine_forkchoiceUpdatedV3",
        json!([final_forkchoice_state_json, serde_json::Value::Null])
    ).await?;
    
    println!("✅ 最终 ForkchoiceUpdated 响应: {}", serde_json::to_string_pretty(&final_forkchoice_result)?);
    
    // 10. 验证余额变化（如果之前获取了余额）
    if let Some(balance_before_wei) = balance_before {
        println!("\n💰 验证余额变化...");
        
        // 等待一下让状态更新
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        match make_http_rpc_call(
            &client,
            "eth_getBalance",
            json!([format!("{:?}", target_address), "latest"])
        ).await {
            Ok(balance_after) => {
                let balance_after_wei = u128::from_str_radix(
                    balance_after.as_str().unwrap_or("0x0").trim_start_matches("0x"),
                    16
                )?;
                let balance_after_eth = balance_after_wei as f64 / 1e18;
                
                println!("新余额: {} ETH ({} Wei)", balance_after_eth, balance_after_wei);
                
                let balance_change_wei = balance_after_wei as i128 - balance_before_wei as i128;
                let balance_change_eth = balance_change_wei as f64 / 1e18;
                
                if balance_change_wei > 0 {
                    println!("✅ 余额增加: {} ETH ({} Wei)", balance_change_eth, balance_change_wei);
                    println!("🎉 成功通过提款机制增加账户余额！");
                } else {
                    println!("⚠️  余额未发生预期变化");
                    println!("   可能需要等待更多时间或检查节点状态");
                }
            },
            Err(e) => {
                println!("⚠️  无法验证余额变化: {}", e);
            }
        }
    }
    
    // 11. 验证新区块
    let updated_latest_block = make_rpc_call(&client, &jwt, "eth_getBlockByNumber", json!(["latest", false])).await?;
    let updated_number = u64::from_str_radix(
        updated_latest_block["number"].as_str().unwrap_or("0x0").trim_start_matches("0x"),
        16
    )?;
    
    if updated_number > current_number {
        println!("\n🎉 成功出块！");
        println!("   原区块: #{} -> 新区块: #{}", current_number, updated_number);
        
        // 检查新区块中的提款
        if let Some(withdrawals_hash) = updated_latest_block.get("withdrawalsRoot") {
            println!("   提款根哈希: {}", withdrawals_hash.as_str().unwrap_or("unknown"));
        }
    }
    
    // 12. 最终余额查询（确保显示最新余额）
    println!("\n💰 最终余额查询...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // 等待更长时间确保状态完全更新
    println!("⏳ 等待 3 秒确保状态更新...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    
    // 查询最终余额
    match make_http_rpc_call(
        &client,
        "eth_getBalance",
        json!([format!("{:?}", target_address), "latest"])
    ).await {
        Ok(final_balance) => {
            let final_balance_wei = u128::from_str_radix(
                final_balance.as_str().unwrap_or("0x0").trim_start_matches("0x"),
                16
            )?;
            let final_balance_eth = final_balance_wei as f64 / 1e18;
            
            println!("📍 地址: {}", target_address);
            println!("💵 最终余额: {} ETH", final_balance_eth);
            println!("   Wei: {}", final_balance_wei);
            println!("   Hex: {}", final_balance.as_str().unwrap_or("0x0"));
            
            // 如果有初始余额，计算变化
            if let Some(initial_wei) = balance_before {
                let change_wei = final_balance_wei as i128 - initial_wei as i128;
                let change_eth = change_wei as f64 / 1e18;
                
                println!("\n📊 余额变化总结:");
                println!("   初始: {} ETH", initial_wei as f64 / 1e18);
                println!("   最终: {} ETH", final_balance_eth);
                println!("   变化: {}{} ETH", if change_wei >= 0 { "+" } else { "" }, change_eth);
                
                if change_wei == 1_000_000_000_000_000_000 {
                    println!("   ✅ 正好增加了 1 ETH！提款成功！");
                } else if change_wei > 0 {
                    println!("   ✅ 余额增加了，但不是预期的 1 ETH");
                } else if change_wei == 0 {
                    println!("   ⚠️  余额没有变化，提款可能还未生效");
                } else {
                    println!("   ❌ 余额减少了，这不应该发生");
                }
            }
        },
        Err(e) => {
            println!("❌ 无法查询最终余额: {}", e);
            println!("   请确保 HTTP RPC 在端口 8545 上可用");
            println!("   可以手动查询: curl -X POST http://127.0.0.1:8545 -H \"Content-Type: application/json\" -d '{{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"{:?}\",\"latest\"],\"id\":1}}'", target_address);
        }
    }
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    println!("\n📋 完整流程总结:");
    println!("1. ✅ 通过 engine_forkchoiceUpdated 请求构建包含提款的载荷");
    println!("2. ✅ 通过 engine_getPayload 获取构建的载荷"); 
    println!("3. ✅ 通过 engine_newPayload 验证载荷");
    println!("4. ✅ 通过 engine_forkchoiceUpdated 实际出块");
    println!("5. ✅ 目标地址应该增加了 1 ETH（通过提款机制）");
    println!("\n这就是通过共识层提款机制增加账户余额的完整流程！");
    
    Ok(())
}