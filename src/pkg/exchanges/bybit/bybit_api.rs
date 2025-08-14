use anyhow::{Result, anyhow};
use async_trait::async_trait;


use crate::pkg::exchanges::bybit::rate_limited_client::RateLimitedClient;
use crate::pkg::exchanges::exchange::ExchangeApi;
use crate::pkg::exchanges::exchange_entities::{BybitTickerInfo, TickerInfo};

#[derive(Debug, serde::Deserialize)]
struct BybitResponse {
    #[serde(rename = "retCode")]
    ret_code: i32,
    #[serde(rename = "retMsg")]
    ret_msg: String,
    result: BybitResult,
}

#[derive(Debug, serde::Deserialize)]
struct BybitResult {
    //category: String,
    list: Vec<BybitTickerInfo>,
}

pub struct BybitApi {
    client: RateLimitedClient,
}

impl BybitApi {
    pub fn new() -> Self {
        Self {
            client: RateLimitedClient::new(None),
        }
    }
}

#[async_trait]
impl ExchangeApi for BybitApi {
    async fn get_all_tickers(&self) -> Result<Vec<TickerInfo>> {
        let url = "https://api.bybit.com/v5/market/tickers?category=linear";
        let req = reqwest::Client::new()
            .get(url)
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        let resp = self.client.send_with_retry(req, 1).await?;
        let status = resp.status();
        let bytes = resp.bytes().await?;

        if !status.is_success() {
            let body = String::from_utf8_lossy(&bytes);
            return Err(anyhow!("Non-200 response: {} - {}", status.as_u16(), body));
        }

        let parsed: BybitResponse = serde_json::from_slice(&bytes)?;
        if parsed.ret_code != 0 {
            return Err(anyhow!("Bybit API error: {}", parsed.ret_msg));
        }

        /*let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();*/

        let standard_tickers = parsed
            .result
            .list
            .into_iter()
            .map(|b| TickerInfo {
                symbol: b.symbol,
                last_price: b.last_price,
                vol_24h: Some(b.volume_24h),
            })
            .collect();

        Ok(standard_tickers)
    }

    /*fn name(&self) -> &str {
        "Bybit"
    }*/

    /*pub async fn get_ticker_info(&self, symbol: &str) -> Result<BybitTickerInfo> {
        let url = format!(
            "https://api.bybit.com/v5/market/ticker?category=linear&symbol={}",
            symbol
        );
        let req = reqwest::Client::new()
            .get(&url)
            .timeout(Duration::from_secs(10))
            .build()?;

        let ticker: BybitTickerInfo = self
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
