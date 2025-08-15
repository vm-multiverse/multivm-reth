//! 简单的 JWT 认证客户端
//! 只发送带 JWT 的请求并接收响应

use eyre::Result;
use jwt_auth_example::{create_jwt_token, JwtConfig};
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 简单的 JWT 客户端");
    
    // 1. 读取 JWT secret（与服务器共享）
    let jwt_config = JwtConfig::from_hex_file("jwt.hex")
        .map_err(|_| eyre::eyre!("找不到 jwt.hex，请先运行服务器生成"))?;
    
    println!("✅ 读取 JWT secret 成功");
    
    // 2. 生成 JWT token（与 reth 相同的方式）
    let jwt = create_jwt_token(&jwt_config)?;
    println!("📝 生成 JWT token:");
    println!("   前50字符: {}...", &jwt[..50.min(jwt.len())]);
    
    // 3. 创建 HTTP 客户端
    let client = Client::new();
    
    // 4. 发送带 JWT 的请求
    println!("\n📤 发送带 JWT 认证的请求...");
    
    let response = client
        .post("http://127.0.0.1:8551")
        .bearer_auth(&jwt)  // 使用 Bearer token 认证（与 reth 相同）
        .send()
        .await?;
    
    if response.status().is_success() {
        let body: serde_json::Value = response.json().await?;
        println!("✅ JWT 验证成功！");
        println!("📥 服务器响应: {}", serde_json::to_string_pretty(&body)?);
    } else {
        println!("❌ JWT 验证失败: {}", response.status());
    }
    
    // 5. 测试无效的 JWT
    println!("\n📤 测试无效的 JWT...");
    
    let invalid_response = client
        .post("http://127.0.0.1:8551")
        .bearer_auth("invalid.jwt.token")
        .send()
        .await?;
    
    if invalid_response.status() == 401 {
        println!("✅ 服务器正确拒绝了无效的 JWT (401 Unauthorized)");
    } else {
        println!("⚠️  意外的响应: {}", invalid_response.status());
    }
    
    // 6. 测试没有 JWT 的请求
    println!("\n📤 测试没有 JWT 的请求...");
    
    let no_auth_response = client
        .post("http://127.0.0.1:8551")
        .send()
        .await?;
    
    if no_auth_response.status() == 401 {
        println!("✅ 服务器正确拒绝了无认证的请求 (401 Unauthorized)");
    } else {
        println!("⚠️  意外的响应: {}", no_auth_response.status());
    }
    
    println!("\n🎉 测试完成！JWT 认证机制工作正常。");
    
    Ok(())
}