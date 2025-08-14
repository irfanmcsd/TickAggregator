use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use log::{error, warn};
use rand::Rng;
use reqwest::{Client, Request, Response};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
struct RateLimitInfo {
    used: i32,
    limit: i32,
    window: Duration,
    reset_time: Instant,
}

impl RateLimitInfo {
    fn should_delay(&self, weight: i32) -> bool {
        let remaining = self.limit - self.used - weight;
        let buffer = (self.limit as f64 * 0.1) as i32;
        remaining <= buffer
    }

    fn get_delay(&self, weight: i32) -> Duration {
        let remaining = self.limit - self.used - weight;
        if remaining >= 0 {
            return Duration::ZERO;
        }
        let elapsed = self.reset_time.saturating_duration_since(Instant::now());
        let per_second = self.limit as f64 / self.window.as_secs_f64();
        let estimate = (-remaining as f64) / per_second;
        let delay_secs = estimate.max(elapsed.as_secs_f64());
        Duration::from_secs_f64(delay_secs)
    }
}

pub struct RateLimitedClient {
    http_client: Client,
    rate_limits: Arc<Mutex<HashMap<String, RateLimitInfo>>>,
    max_retries: usize,
    default_wait: Duration,
}

impl RateLimitedClient {
    pub fn new(client: Option<Client>) -> Self {
        let http_client = client.unwrap_or_else(Client::new);
        Self {
            http_client,
            rate_limits: Arc::new(Mutex::new(HashMap::new())),
            max_retries: 5,
            default_wait: Duration::from_secs(5),
        }
    }

    pub async fn send_with_retry(
        &self,
        req: Request,
        weight: i32,
    ) -> Result<Response, reqwest::Error> {
        let mut retry_count = 0;

        loop {
            self.apply_rate_limiting(req.url().path(), weight).await;

            let resp_result = self.http_client.execute(req.try_clone().expect("Failed to clone request")).await;

            match resp_result {
                Ok(resp) => {
                    self.update_rate_limits(resp.headers()).await;

                    if resp.status().as_u16() == 429 || resp.status().as_u16() == 418 {
                        if retry_count >= self.max_retries {
                            error!("Max retries reached for rate limited response");
                            return Ok(resp);
                        }
                        let delay = self.get_retry_delay(Some(&resp), retry_count).await;
                        warn!("Rate limited, retrying after {:?}", delay);
                        tokio::time::sleep(delay).await;
                        retry_count += 1;
                        continue;
                    }
                    return Ok(resp);
                }
                Err(e) => {
                    if is_transient(&e) {
                        if retry_count >= self.max_retries {
                            error!("Max retries reached for transient error: {}", e);
                            return Err(e);
                        }
                        let delay = self.get_retry_delay(None, retry_count).await;
                        warn!("Transient error: {}. Retrying after {:?}", e, delay);
                        tokio::time::sleep(delay).await;
                        retry_count += 1;
                        continue;
                    }
                    return Err(e);
                }
            }
        }
    }

    async fn apply_rate_limiting(&self, key: &str, weight: i32) {
        let mut rate_limits = self.rate_limits.lock().await;

        if let Some(info) = rate_limits.get(key) {
            if info.should_delay(weight) {
                let delay = info.get_delay(weight);
                if delay > Duration::ZERO {
                    warn!("Delaying request for {:?} due to rate limit", delay);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    async fn update_rate_limits(&self, headers: &reqwest::header::HeaderMap) {
        let header_mapping = vec![
            ("X-MBX-USED-WEIGHT-1M", "weight", Duration::from_secs(60), 1200),
            ("X-MBX-ORDER-COUNT-1M", "orders", Duration::from_secs(60), 1600),
            ("X-MBX-ORDER-COUNT-1D", "daily_orders", Duration::from_secs(60 * 60 * 24), 10000),
        ];

        let mut rate_limits = self.rate_limits.lock().await;

        for (header_name, key, window, default_limit) in header_mapping {
            if let Some(value) = headers.get(header_name) {
                if let Ok(used_str) = value.to_str() {
                    if let Ok(used) = used_str.parse::<i32>() {
                        rate_limits.insert(
                            key.to_string(),
                            RateLimitInfo {
                                used,
                                limit: default_limit,
                                window,
                                reset_time: Instant::now() + window,
                            },
                        );
                    }
                }
            }
        }
    }

    async fn get_retry_delay(&self, resp: Option<&Response>, retry: usize) -> Duration {
        if let Some(resp) = resp {
            if let Some(val) = resp.headers().get("Retry-After") {
                if let Ok(s) = val.to_str() {
                    if let Ok(seconds) = s.parse::<u64>() {
                        return Duration::from_secs(seconds);
                    }
                }
            }
        }
        let base = 2_f64.powf(retry as f64);
        let jitter: f64 = rand::thread_rng().gen_range(0.75..1.25);
        Duration::from_secs_f64(base * jitter)
    }
}

fn is_transient(err: &reqwest::Error) -> bool {
    if err.is_timeout() {
        return true;
    }
    let err_str = err.to_string().to_lowercase();
    err_str.contains("timeout") || err_str.contains("connection reset")
}
