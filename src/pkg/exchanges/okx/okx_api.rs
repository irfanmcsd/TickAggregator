use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde::Deserialize;

use crate::pkg::exchanges::exchange::ExchangeApi;
use crate::pkg::exchanges::exchange_entities::{OKXTickerInfo, TickerInfo};
use crate::pkg::exchanges::okx::rate_limited_client::RateLimitedClient;


#[derive(Debug, Deserialize)]
struct OkxResponse<T> {
    code: String,
    msg: String,
    data: T,
}
pub struct OkxApi {
    client: RateLimitedClient,
}

impl OkxApi {
    pub fn new() -> Self {
        Self {
            client: RateLimitedClient::new(None),
        }
    }
}

#[async_trait]
impl ExchangeApi for OkxApi {
    async fn get_all_tickers(&self) -> Result<Vec<TickerInfo>> {
        let url = "https://www.okx.com/api/v5/market/tickers?instType=SWAP";
        let req = reqwest::Client::new()
            .get(url)
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        let resp = self.client.send_with_retry(req, 1).await?;
        let status = resp.status();
        let bytes = resp.bytes().await?;

        if !status.is_success() {
            let body = String::from_utf8_lossy(&bytes);
            return Err(anyhow!(
                "Non-200 response: {} - {}",
                status.as_u16(),
                body
            ));
        }

        // Parse into wrapper struct
        let parsed: OkxResponse<Vec<OKXTickerInfo>> = serde_json::from_slice(&bytes)?;

        if parsed.code != "0" {
            return Err(anyhow!("OKX API error: {} - {}", parsed.code, parsed.msg));
        }

        let standard_tickers = parsed
            .data
            .into_iter()
            .map(|o| TickerInfo {
                symbol: o.instrument_id,
                last_price: o.last_price,
                vol_24h: Some(o.volume_24h),
            })
            .collect();

        Ok(standard_tickers)
    }

    /*fn name(&self) -> &str {
        "OKX"
    }*/

    /*pub async fn get_ticker_info(&self, inst_id: &str) -> Result<OKXTickerInfo> {
        let url = format!(
            "https://www.okx.com/api/v5/market/ticker?instId={}",
            inst_id
        );
        let req = reqwest::Client::new()
            .get(&url)
            .timeout(Duration::from_secs(10))
            .build()?;

        let ticker: OKXTickerInfo = self
            .client
            .send_with_retry(req, 1)
            .await
            .context("Send failed")?
            .error_for_status()
            .context("Bad status")?
            .json()
            .await
            .context("JSON parse failed")?;

        Ok(ticker)
    }*/
}
