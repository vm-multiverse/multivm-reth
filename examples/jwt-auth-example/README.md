# JWT 认证示例

这个示例展示了与 reth 相同的 JWT 认证机制。

## 快速开始

```bash
# 终端 1：启动服务器
cargo run --bin server

# 终端 2：运行客户端
cargo run --bin client
```

## 工作原理

1. **服务器和客户端共享密钥** (`jwt.hex`)
2. **客户端生成 JWT token**（包含 iat 和 exp）
3. **客户端发送请求**（使用 Bearer token）
4. **服务器验证 JWT** 并返回响应

## 核心代码

### 生成 JWT（与 reth 相同）
```rust
let claims = Claims {
    iat: now,           // 签发时间
    exp: now + 3600,    // 过期时间（1小时）
};
let token = encode(&Header::new(Algorithm::HS256), &claims, &key)?;
```

### 发送请求（客户端）
```rust
client.post("http://127.0.0.1:8551")
    .bearer_auth(&jwt)  // Bearer 认证
    .send()
```

### 验证 JWT（服务器）
```rust
let token = extract_bearer_token(auth_header)?;
let claims = validate_jwt_token(&token, &jwt_config)?;
```

## 文件说明

- `src/lib.rs` - JWT 工具库（创建、验证 token）
- `src/simple_server.rs` - 服务器（验证 JWT）
- `src/simple_client.rs` - 客户端（发送带 JWT 的请求）