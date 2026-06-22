use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 1000,
            max_delay_ms: 30000,
        }
    }
}

pub async fn with_retry<T, F, Fut>(config: &RetryConfig, mut operation: F) -> Result<T, String>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, String>>,
{
    let mut last_err = String::new();
    for attempt in 0..config.max_attempts {
        match operation().await {
            Ok(val) => return Ok(val),
            Err(e) => {
                last_err = e;
                if attempt + 1 < config.max_attempts && is_retryable_error(&last_err) {
                    let delay = config.base_delay_ms.saturating_mul(1u64 << attempt);
                    let delay = delay.min(config.max_delay_ms);
                    log::warn!(
                        "Retry attempt {}/{} after {}ms: {}",
                        attempt + 1,
                        config.max_attempts,
                        delay,
                        last_err
                    );
                    sleep(Duration::from_millis(delay)).await;
                }
            }
        }
    }
    Err(last_err)
}

pub fn is_retryable_error(error: &str) -> bool {
    let lower = error.to_lowercase();
    lower.contains("429")
        || lower.contains("500")
        || lower.contains("502")
        || lower.contains("503")
        || lower.contains("timeout")
        || lower.contains("timed out")
        || lower.contains("connection")
        || lower.contains("network")
        || lower.contains("eof")
        || lower.contains("broken pipe")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn succeeds_first_try() {
        let config = RetryConfig::default();
        let result = with_retry(&config, || async { Ok::<_, String>(42) }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn succeeds_after_retryable_failure() {
        let attempts = Arc::new(AtomicU32::new(0));
        let config = RetryConfig {
            max_attempts: 3,
            base_delay_ms: 10,
            max_delay_ms: 50,
        };
        let attempts_clone = attempts.clone();
        let result = with_retry(&config, move || {
            let a = attempts_clone.clone();
            async move {
                let count = a.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err("429 Too Many Requests".to_string())
                } else {
                    Ok(99)
                }
            }
        })
        .await;
        assert_eq!(result.unwrap(), 99);
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn exhausts_attempts() {
        let attempts = Arc::new(AtomicU32::new(0));
        let config = RetryConfig {
            max_attempts: 3,
            base_delay_ms: 10,
            max_delay_ms: 50,
        };
        let attempts_clone = attempts.clone();
        let result: Result<(), String> = with_retry(&config, move || {
            let a = attempts_clone.clone();
            async move {
                a.fetch_add(1, Ordering::SeqCst);
                Err("429 rate limited".to_string())
            }
        })
        .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "429 rate limited");
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn is_retryable_classifies_correctly() {
        assert!(is_retryable_error("429 Too Many Requests"));
        assert!(is_retryable_error("HTTP 500 Internal Server Error"));
        assert!(is_retryable_error("502 Bad Gateway"));
        assert!(is_retryable_error("503 Service Unavailable"));
        assert!(is_retryable_error("connection timed out"));
        assert!(is_retryable_error("NetworkError: timeout"));
        assert!(is_retryable_error("connection refused"));
        assert!(is_retryable_error("broken pipe"));
        assert!(!is_retryable_error("400 Bad Request"));
        assert!(!is_retryable_error("401 Unauthorized"));
        assert!(!is_retryable_error("404 Not Found"));
        assert!(!is_retryable_error("invalid json"));
    }
}
