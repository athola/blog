use std::time::Duration;

use tokio_retry::{strategy::ExponentialBackoff, Retry};
use tracing::{error, warn};

/// Configuration for retry behavior when invoking asynchronous operations.
#[derive(Debug, Clone, Copy)]
pub struct RetryConfig {
    pub initial_delay_millis: u64,
    pub max_delay_secs: u64,
    pub max_retries: u32,
}

impl RetryConfig {
    #[must_use]
    pub fn new(initial_delay_millis: u64, max_delay_secs: u64, max_retries: u32) -> Self {
        Self {
            initial_delay_millis,
            max_delay_secs,
            max_retries,
        }
    }

    fn strategy(&self) -> impl Iterator<Item = Duration> + Clone {
        ExponentialBackoff::from_millis(self.initial_delay_millis)
            .max_delay(Duration::from_secs(self.max_delay_secs))
            .take(self.max_retries as usize)
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            initial_delay_millis: 50,
            max_delay_secs: 2,
            max_retries: 3,
        }
    }
}

/// Execute an asynchronous operation with retry/backoff semantics.
///
/// `context` is included in log messages to provide call-site visibility.
pub async fn retry_async<F, Fut, T, E>(
    context: &str,
    config: RetryConfig,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let strategy = config.strategy();
    let result = Retry::spawn(strategy, || {
        let fut = operation();
        async move {
            match fut.await {
                Ok(value) => Ok(value),
                Err(err) => {
                    warn!(error = ?err, retry_context = context, "Operation failed; retrying");
                    Err(err)
                }
            }
        }
    })
    .await;

    if let Err(err) = &result {
        error!(
            error = ?err,
            retry_context = context,
            "Operation failed after exhausting retries"
        );
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn succeeds_after_retries() {
        tokio_test::block_on(async {
            let attempts = Arc::new(AtomicUsize::new(0));
            let tracker = attempts.clone();

            let result = retry_async("test_success", RetryConfig::default(), move || {
                let tracker = tracker.clone();
                async move {
                    let current = tracker.fetch_add(1, Ordering::SeqCst);
                    if current < 2 {
                        Err::<_, &'static str>("fail")
                    } else {
                        Ok::<_, &'static str>("ok")
                    }
                }
            })
            .await;

            assert_eq!(result.unwrap(), "ok");
            assert_eq!(attempts.load(Ordering::SeqCst), 3);
        });
    }

    #[test]
    fn returns_error_after_exhausting_retries() {
        tokio_test::block_on(async {
            let attempts = Arc::new(AtomicUsize::new(0));
            let tracker = attempts.clone();

            let config = RetryConfig::default();
            let result: Result<(), &str> = retry_async("test_failure", config, move || {
                let tracker = tracker.clone();
                async move {
                    tracker.fetch_add(1, Ordering::SeqCst);
                    Err("nope")
                }
            })
            .await;

            assert!(result.is_err());
            assert_eq!(
                attempts.load(Ordering::SeqCst),
                config.max_retries as usize + 1
            );
        });
    }

    #[test]
    fn honors_custom_config() {
        tokio_test::block_on(async {
            let config = RetryConfig::new(10, 1, 5);
            let attempts = Arc::new(AtomicUsize::new(0));
            let tracker = attempts.clone();

            let _ = retry_async("custom_config", config, move || {
                let tracker = tracker.clone();
                async move {
                    tracker.fetch_add(1, Ordering::SeqCst);
                    Err::<(), &str>("fail")
                }
            })
            .await;

            assert_eq!(
                attempts.load(Ordering::SeqCst),
                config.max_retries as usize + 1
            );
        });
    }
}
