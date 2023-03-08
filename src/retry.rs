use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub struct RetryPolicy {
    pub max_retries: u64,
    pub jitter: bool,
    pub retry_on_client_error: bool,
    pub retry_on_server_error: bool,
    pub timeout: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            jitter: true,
            retry_on_client_error: false,
            retry_on_server_error: true,
            timeout: Duration::from_secs(1),
        }
    }
}
