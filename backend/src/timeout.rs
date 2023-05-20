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
    pub async fn execute<F, C, R>(
        self,
        closure: C,
        shared_database: SharedRemoteDatabase,
    ) -> ApiResult<R>
    where
        C: Fn(SharedRemoteDatabase) -> F,
        F: Future<Output = R>,
    {
        match self {
            Self::LinearRetry {
                times: max_times,
                sleep,
            } => {
                let mut retried_times = 0;

                loop {
                    let reconfiguration_id = shared_database.read().await.reconfiguration_id();
                    match timeout_in(sleep, closure(Arc::clone(&shared_database))).await {
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
                                let mut shared_database = shared_database.write().await;
                                if shared_database.reconfiguration_id() == reconfiguration_id {
                                    tracing::info!("Reconfiguring... {shared_database:?}");

                                    match shared_database.reconfigure().await {
                                        Ok(()) => {
                                            tracing::info!("Reconfigured {shared_database:?}")
                                        }
                                        Err(e) => {
                                            tracing::info!(
                                                "Failed to reconfigure {shared_database:?}: {e}",
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
    startup::SharedRemoteDatabase,
};
use std::future::Future;
use std::{sync::Arc, time::Duration};
use tokio::time::timeout as timeout_in;
