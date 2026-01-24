// API Key 认证中间件
use axum::{
    extract::Request,
    extract::State,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::proxy::config::ApiKeyEntry;
use crate::proxy::{ProxyAuthMode, ProxySecurityConfig};

/// 允许访问的账号列表（存储在 Request Extensions 中）
#[derive(Debug, Clone)]
pub struct AllowedAccounts(pub Vec<String>);

/// API Key 认证中间件
pub async fn auth_middleware(
    State(security): State<Arc<RwLock<ProxySecurityConfig>>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let method = request.method().clone();
    let path = request.uri().path().to_string();

    // 过滤心跳和健康检查请求,避免日志噪音
    if !path.contains("event_logging") && path != "/healthz" {
        tracing::info!("Request: {} {}", method, path);
    } else {
        tracing::trace!("Heartbeat: {} {}", method, path);
    }

    // Allow CORS preflight regardless of auth policy.
    if method == axum::http::Method::OPTIONS {
        return Ok(next.run(request).await);
    }

    let security = security.read().await.clone();
    let effective_mode = security.effective_auth_mode();

    if matches!(effective_mode, ProxyAuthMode::Off) {
        return Ok(next.run(request).await);
    }

    if matches!(effective_mode, ProxyAuthMode::AllExceptHealth) && path == "/healthz" {
        return Ok(next.run(request).await);
    }

    // 从 header 中提取 API key
    let api_key = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer ").or(Some(s)))
        .or_else(|| {
            request
                .headers()
                .get("x-api-key")
                .and_then(|h| h.to_str().ok())
        })
        .or_else(|| {
            request
                .headers()
                .get("x-goog-api-key")
                .and_then(|h| h.to_str().ok())
        });

    if security.api_keys.is_empty() {
        tracing::error!("Proxy auth is enabled but api_keys is empty; denying request");
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Check if the provided key matches any of the configured keys
    let matched_entry =
        api_key.and_then(|k| security.api_keys.iter().find(|entry| entry.key() == k));

    if let Some(entry) = matched_entry {
        let mut request = request;
        // Always insert the extension, either Some(accounts) or None
        let allowed_accounts = match entry {
            ApiKeyEntry::Mapped { accounts, .. } if !accounts.is_empty() => {
                Some(AllowedAccounts(accounts.clone()))
            }
            _ => None,
        };
        request.extensions_mut().insert(allowed_accounts);
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

#[cfg(test)]
mod tests {
    // 移除未使用的 use super::*;

    #[test]
    fn test_auth_placeholder() {
        // Placeholder test
        assert!(true);
    }
}
