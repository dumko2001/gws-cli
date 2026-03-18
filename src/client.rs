use reqwest::header::{HeaderMap, HeaderValue};

pub fn build_client() -> Result<reqwest::Client, crate::error::GwsError> {
    let mut headers = HeaderMap::new();
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");

    // Format: gl-rust/name-version (the gl-rust/ prefix is fixed)
    let client_header = format!("gl-rust/{}-{}", name, version);
    if let Ok(header_value) = HeaderValue::from_str(&client_header) {
        headers.insert("x-goog-api-client", header_value);
    }

    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| {
            crate::error::GwsError::Other(anyhow::anyhow!("Failed to build HTTP client: {e}"))
        })
}

const MAX_RETRIES: u32 = 3;
/// Maximum seconds to sleep on a 429 Retry-After header. Prevents a hostile
/// or misconfigured server from hanging the process indefinitely.
const MAX_RETRY_DELAY_SECS: u64 = 60;

/// Send an HTTP request with automatic retry on 429 (rate limit) and transient 5xx responses.
/// Respects the `Retry-After` header; falls back to exponential backoff (1s, 2s, 4s).
pub async fn send_with_retry<F>(build_request: F) -> anyhow::Result<reqwest::Response>
where
    F: Fn() -> anyhow::Result<reqwest::RequestBuilder>,
{
    for attempt in 0..MAX_RETRIES {
        let req_result = build_request()?.send().await;

        let resp = match req_result {
            Ok(r) => r,
            Err(e) if attempt < MAX_RETRIES - 1 && (e.is_timeout() || e.is_connect()) => {
                let retry_after = compute_retry_delay(None, attempt);
                tracing::debug!(error = %e, attempt, retry_after, "Retrying on network error");
                tokio::time::sleep(std::time::Duration::from_secs(retry_after)).await;
                continue;
            }
            Err(e) => return Err(e.into()),
        };

        let status = resp.status();
        if status.is_success()
            || !matches!(
                status,
                reqwest::StatusCode::TOO_MANY_REQUESTS
                    | reqwest::StatusCode::INTERNAL_SERVER_ERROR
                    | reqwest::StatusCode::BAD_GATEWAY
                    | reqwest::StatusCode::SERVICE_UNAVAILABLE
            )
        {
            return Ok(resp);
        }

        // If we still have retries, sleep and continue
        let header_value = resp
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok());
        let retry_after = compute_retry_delay(header_value, attempt);

        tracing::debug!(status = %status, attempt, retry_after, "Retrying on server error");
        tokio::time::sleep(std::time::Duration::from_secs(retry_after)).await;
    }

    // Final attempt — return whatever we get
    Ok(build_request()?.send().await?)
}

/// Compute the retry delay from a Retry-After header value and attempt number.
/// Falls back to exponential backoff (1, 2, 4s) when the header is absent or
/// unparseable. Always caps the result at MAX_RETRY_DELAY_SECS.
fn compute_retry_delay(header_value: Option<&str>, attempt: u32) -> u64 {
    header_value
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(2u64.saturating_pow(attempt))
        .min(MAX_RETRY_DELAY_SECS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_client_succeeds() {
        assert!(build_client().is_ok());
    }

    #[test]
    fn retry_delay_caps_large_header_value() {
        assert_eq!(compute_retry_delay(Some("999999"), 0), MAX_RETRY_DELAY_SECS);
    }

    #[test]
    fn retry_delay_passes_through_small_header_value() {
        assert_eq!(compute_retry_delay(Some("5"), 0), 5);
    }

    #[test]
    fn retry_delay_falls_back_to_exponential_on_missing_header() {
        assert_eq!(compute_retry_delay(None, 0), 1); // 2^0
        assert_eq!(compute_retry_delay(None, 1), 2); // 2^1
        assert_eq!(compute_retry_delay(None, 2), 4); // 2^2
    }

    #[test]
    fn retry_delay_falls_back_on_unparseable_header() {
        assert_eq!(compute_retry_delay(Some("not-a-number"), 1), 2);
        assert_eq!(compute_retry_delay(Some(""), 0), 1);
    }

    #[test]
    fn retry_delay_caps_at_boundary() {
        assert_eq!(compute_retry_delay(Some("60"), 0), 60);
        assert_eq!(compute_retry_delay(Some("61"), 0), MAX_RETRY_DELAY_SECS);
    }
}
