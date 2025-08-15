//! 简单的 JWT 认证服务器
//! 只验证 JWT 并返回一个字符串

use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::Json,
    routing::post,
    Router,
};
use jwt_auth_example::{create_jwt_token, extract_bearer_token, validate_jwt_token, JwtConfig};
use serde_json::json;
use std::sync::Arc;

#[derive(Clone)]
struct ServerState {
    jwt_config: Arc<JwtConfig>,
}

/// 简单的处理函数：验证 JWT 并返回消息
async fn handle_request(
    State(state): State<ServerState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, StatusCode> {
    println!("\n📥 收到新请求");
    
    // 1. 提取 Bearer token
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());
    
    match auth_header {
        Some(header) => {
            println!("   发现 Authorization header");
            match extract_bearer_token(header) {
                Some(token) => {
                    println!("   提取到 Bearer token: {}...", &token[..20.min(token.len())]);
                    
                    // 2. 验证 JWT
                    match validate_jwt_token(&token, &state.jwt_config) {
                        Ok(claims) => {
                            println!("   ✅ JWT 验证成功!");
                            println!("      - 签发时间: {}", claims.iat);
                            println!("      - 过期时间: {}", claims.exp);
                            
                            // 3. JWT 验证成功，返回简单消息
                            Ok(Json(json!({
                                "message": "Hello! JWT validation successful",
                                "jwt_issued_at": claims.iat,
                                "jwt_expires_at": claims.exp
                            })))
                        }
                        Err(e) => {
                            println!("   ❌ JWT 验证失败: {}", e);
                            Err(StatusCode::UNAUTHORIZED)
                        }
                    }
                }
                None => {
                    println!("   ❌ 无效的 Bearer token 格式");
                    Err(StatusCode::UNAUTHORIZED)
                }
            }
        }
        None => {
            println!("   ❌ 缺少 Authorization header");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    println!("🚀 启动简单的 JWT 服务器");
    
    // 读取或生成 JWT secret
    let jwt_secret_path = "jwt.hex";
    let jwt_config = if std::path::Path::new(jwt_secret_path).exists() {
        println!("✅ 使用现有 JWT secret: {}", jwt_secret_path);
        JwtConfig::from_hex_file(jwt_secret_path)?
    } else {
        println!("📝 生成新的 JWT secret");
        let secret = jwt_auth_example::generate_jwt_secret();
        std::fs::write(jwt_secret_path, &secret)?;
        JwtConfig::from_hex_string(&secret)?
    };
    
    // 生成示例 token
    let example_token = create_jwt_token(&jwt_config)?;
    println!("\n📋 示例 JWT token:");
    println!("{}", example_token);
    println!("\n测试命令:");
    println!("curl -H \"Authorization: Bearer {}\" http://localhost:8551/", example_token);
    
    let state = ServerState {
        jwt_config: Arc::new(jwt_config),
    };
    
    let app = Router::new()
        .route("/", post(handle_request))
        .with_state(state);
    
    println!("\n🌐 服务器运行在: http://127.0.0.1:8551");
    
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8551").await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}