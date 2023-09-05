// Timeouts for closures that don't need to change state when retrying
pub enum TimeoutStrategy {
    Once { timeout: Duration },
}

impl Default for TimeoutStrategy {
    fn default() -> Self {
        Self::Once {
            timeout: Duration::from_secs(3),
        }
    }
}

impl TimeoutStrategy {
    pub async fn execute<F, C, R>(self, closure: C) -> ApiResult<R>
    where
        C: Fn() -> F,
        F: Future<Output = R>,
    {
        match self {
            Self::Once { timeout } => match timeout_in(timeout, closure()).await {
                Ok(v) => Ok(v),
                Err(_) => Err(ApiError::FutureTimeout),
            },
        }
    }
}

use crate::error::{ApiError, ApiResult};
use std::future::Future;
use std::time::Duration;
use tokio::time::timeout as timeout_in;
