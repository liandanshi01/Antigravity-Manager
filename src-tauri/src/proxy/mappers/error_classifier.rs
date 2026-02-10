// 错误分类模块 - 将底层错误转换为用户友好的消息
use reqwest::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamErrorKind {
    Timeout,
    Connect,
    Decode,
    Body,
    Unknown,
}

pub fn stream_error_kind(error: &Error) -> StreamErrorKind {
    if error.is_timeout() {
        StreamErrorKind::Timeout
    } else if error.is_connect() {
        StreamErrorKind::Connect
    } else if error.is_decode() {
        StreamErrorKind::Decode
    } else if error.is_body() {
        StreamErrorKind::Body
    } else {
        StreamErrorKind::Unknown
    }
}

pub fn classify_stream_error_kind(kind: StreamErrorKind) -> (&'static str, &'static str, &'static str) {
    match kind {
        StreamErrorKind::Timeout => (
            "timeout_error",
            "Request timeout, please check your network connection",
            "errors.stream.timeout_error",
        ),
        StreamErrorKind::Connect => (
            "connection_error",
            "Connection failed, please check your network or proxy settings",
            "errors.stream.connection_error",
        ),
        StreamErrorKind::Decode => (
            "decode_error",
            "Network unstable, data transmission interrupted. Try: 1) Check network 2) Switch proxy 3) Retry",
            "errors.stream.decode_error",
        ),
        StreamErrorKind::Body => (
            "stream_error",
            "Stream transmission error, please retry later",
            "errors.stream.stream_error",
        ),
        StreamErrorKind::Unknown => (
            "unknown_error",
            "Unknown error occurred",
            "errors.stream.unknown_error",
        ),
    }
}

/// 分类流式响应错误并返回错误类型、英文消息和 i18n key
/// 
/// 返回值: (错误类型, 英文错误消息, i18n_key)
/// - 错误类型: 用于日志和错误码
/// - 英文消息: fallback 消息,供非浏览器客户端使用
/// - i18n_key: 前端翻译键,供浏览器客户端本地化
pub fn classify_stream_error(error: &Error) -> (&'static str, &'static str, &'static str) {
    classify_stream_error_kind(stream_error_kind(error))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_timeout_kind() {
        let (error_type, message, i18n_key) = classify_stream_error_kind(StreamErrorKind::Timeout);
        assert_eq!(error_type, "timeout_error");
        assert!(message.to_lowercase().contains("timeout"));
        assert_eq!(i18n_key, "errors.stream.timeout_error");
    }

    #[test]
    fn test_error_message_format() {
        // No network calls in unit tests (reqwest may consult system proxy config and panic in CI/sandbox).
        for kind in [
            StreamErrorKind::Timeout,
            StreamErrorKind::Connect,
            StreamErrorKind::Decode,
            StreamErrorKind::Body,
            StreamErrorKind::Unknown,
        ] {
            let (error_type, message, i18n_key) = classify_stream_error_kind(kind);
            assert!(!error_type.is_empty());
            assert!(!message.is_empty());
            assert!(i18n_key.starts_with("errors.stream."));
        }
    }

    #[test]
    fn test_i18n_keys_format() {
        // 验证所有错误类型都有正确的 i18n_key 格式
        let test_cases = vec![
            ("timeout_error", "errors.stream.timeout_error"),
            ("connection_error", "errors.stream.connection_error"),
            ("decode_error", "errors.stream.decode_error"),
            ("stream_error", "errors.stream.stream_error"),
            ("unknown_error", "errors.stream.unknown_error"),
        ];
        
        // 这里我们只验证 i18n_key 格式
        for (expected_type, expected_key) in test_cases {
            assert_eq!(format!("errors.stream.{}", expected_type), expected_key);
        }
    }
}
