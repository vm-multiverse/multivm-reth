//! JWT 认证共享模块
//! 
//! 这个模块提供了与 reth 相同的 JWT 认证机制：
//! - 使用 HS256 算法
//! - 包含 iat (issued at) 和 exp (expiration) 声明
//! - 从 hex 编码的密钥文件读取

use eyre::Result;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// JWT Claims 结构，与 reth 保持一致
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// 签发时间 (issued at)
    pub iat: u64,
    /// 过期时间 (expiration)
    pub exp: u64,
}

/// JWT 配置
pub struct JwtConfig {
    /// 密钥（从 hex 文件读取的原始字节）
    pub secret: Vec<u8>,
    /// Token 有效期（秒）
    pub expiry_seconds: u64,
}

impl JwtConfig {
    /// 从 hex 文件创建配置
    pub fn from_hex_file(path: &str) -> Result<Self> {
        let hex_secret = std::fs::read_to_string(path)?
            .trim()
            .to_string();
        let secret = hex::decode(hex_secret)?;
        
        Ok(Self {
            secret,
            expiry_seconds: 3600, // 默认 1 小时
        })
    }
    
    /// 从 hex 字符串创建配置
    pub fn from_hex_string(hex_secret: &str) -> Result<Self> {
        let secret = hex::decode(hex_secret.trim())?;
        Ok(Self {
            secret,
            expiry_seconds: 3600,
        })
    }
}

/// 创建 JWT token（与 reth 的 create_jwt_token 相同）
pub fn create_jwt_token(config: &JwtConfig) -> Result<String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();
    
    let claims = Claims {
        iat: now,
        exp: now + config.expiry_seconds,
    };
    
    let header = Header::new(Algorithm::HS256);
    let key = EncodingKey::from_secret(&config.secret);
    
    let token = encode(&header, &claims, &key)?;
    Ok(token)
}

/// 验证 JWT token
pub fn validate_jwt_token(token: &str, config: &JwtConfig) -> Result<Claims> {
    let key = DecodingKey::from_secret(&config.secret);
    let validation = Validation::new(Algorithm::HS256);
    
    let token_data = decode::<Claims>(token, &key, &validation)?;
    Ok(token_data.claims)
}

/// 从 Authorization header 提取 Bearer token
pub fn extract_bearer_token(auth_header: &str) -> Option<String> {
    if auth_header.starts_with("Bearer ") {
        Some(auth_header[7..].to_string())
    } else {
        None
    }
}

/// 生成新的 JWT secret（32 字节）
pub fn generate_jwt_secret() -> String {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hash, Hasher};
    
    // 使用系统时间和随机状态生成伪随机数
    let mut hasher = RandomState::new().build_hasher();
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .hash(&mut hasher);
    
    let mut secret = Vec::new();
    for i in 0..4 {
        let mut h = hasher.clone();
        i.hash(&mut h);
        let bytes = h.finish().to_be_bytes();
        secret.extend_from_slice(&bytes);
    }
    
    hex::encode(secret)
}

/// RPC 请求结构（与 reth 兼容）
#[derive(Debug, Serialize, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: serde_json::Value,
}

/// RPC 响应结构
#[derive(Debug, Serialize, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

/// RPC 错误结构
#[derive(Debug, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl RpcError {
    pub fn unauthorized() -> Self {
        Self {
            code: -32000,
            message: "Unauthorized: Invalid or missing JWT token".to_string(),
            data: None,
        }
    }
    
    pub fn invalid_token() -> Self {
        Self {
            code: -32001,
            message: "Invalid JWT token".to_string(),
            data: None,
        }
    }
    
    pub fn token_expired() -> Self {
        Self {
            code: -32002,
            message: "JWT token has expired".to_string(),
            data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_jwt_creation_and_validation() {
        let secret = "a".repeat(64); // 32 bytes in hex
        let config = JwtConfig::from_hex_string(&secret).unwrap();
        
        let token = create_jwt_token(&config).unwrap();
        let claims = validate_jwt_token(&token, &config).unwrap();
        
        assert!(claims.exp > claims.iat);
        assert_eq!(claims.exp - claims.iat, config.expiry_seconds);
    }
    
    #[test]
    fn test_bearer_token_extraction() {
        let header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let token = extract_bearer_token(header);
        assert_eq!(token, Some("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9".to_string()));
        
        let invalid_header = "Basic dXNlcjpwYXNz";
        assert_eq!(extract_bearer_token(invalid_header), None);
    }
}