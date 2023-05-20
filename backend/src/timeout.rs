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

// timeout used from handlers for database requests
pub enum HangingStrategy {
    LinearRetry { times: u32, sleep: Duration },
}

impl Default for HangingStrategy {
    fn default() -> Self {
        Self::LinearRetry {
            times: 1,
            sleep: Duration::from_secs(1),
        }
    }
}

impl HangingStrategy {
    pub fn long_linear() -> Self {
        Self::LinearRetry {
            times: 1,
            sleep: Duration::from_secs(7),
        }
    }

    // Attempt to renew connection with the database server if it hangs
    // because there's no timeouts on external API calls to it
    pub async fn execute<F, C, R>(self, closure: C, db_client: SharedDbClient) -> ApiResult<R>
    where
        C: Fn(SharedDbClient) -> F,
        F: Future<Output = R>,
    {
        match self {
            Self::LinearRetry {
                times: max_times,
                sleep,
            } => {
                let mut retried_times = 0;

                loop {
                    let reconfiguration_id = db_client.read().await.reconfiguration_id();
                    match timeout_in(sleep, closure(Arc::clone(&db_client))).await {
                        Ok(r) => return Ok(r),
                        Err(_elapsed) => {
                            if retried_times >= max_times {
                                return Err(ApiError::DatabaseHangs);
                            }

                            // When several requests hang - reconfigure the client once, let others wait
                            // Before deciding whether to reconfigure client -
                            // check the reconfiguration_id of the client the request was tried with
                            // if ID does not match - client has changed, so retry the request
                            {
                                let mut db_client = db_client.write().await;
                                if db_client.reconfiguration_id() == reconfiguration_id {
                                    tracing::info!("Reconfiguring... {db_client:?}");

                                    match db_client.reconfigure().await {
                                        Ok(()) => {
                                            tracing::info!("Reconfigured {db_client:?}")
                                        }
                                        Err(e) => {
                                            tracing::info!(
                                                "Failed to reconfigure {db_client:?}: {e}",
                                            )
                                        }
                                    }

                                    retried_times += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

use crate::{
    error::{ApiError, ApiResult},
    startup::SharedDbClient,
};
use std::future::Future;
use std::{sync::Arc, time::Duration};
use tokio::time::timeout as timeout_in;
