use crate::pkg::exchanges::bitget::rate_limited_client::RateLimitedClient;
use crate::pkg::exchanges::exchange_entities::BitgetTickerInfo;
use crate::pkg::exchanges::exchange_entities::TickerInfo;
use crate::pkg::exchanges::exchange::ExchangeApi;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct BitgetResponse<T> {
    //code: String,
    //msg: String,
    data: T,
}

pub struct BitgetApi {
    client: RateLimitedClient,
}

impl BitgetApi {
    pub fn new() -> Self {
        Self {
            client: RateLimitedClient::new(None),
        }
    }
}


#[async_trait]
impl ExchangeApi for BitgetApi {
     async fn get_all_tickers(&self) -> Result<Vec<TickerInfo>> {
        let url = "https://api.bitget.com/api/mix/v1/market/tickers?productType=umcbl";
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

        // Deserialize into BitgetResponse<Vec<BitgetTickerInfo>>
        let parsed: BitgetResponse<Vec<BitgetTickerInfo>> = serde_json::from_slice(&bytes)?;

        let standard_tickers = parsed
            .data
            .into_iter()
            .map(|b| TickerInfo {
                symbol: b.symbol,
                last_price: b.last_price,
                vol_24h: Some(b.base_volume),
            })
            .collect();

        Ok(standard_tickers)
    }

    /*fn name(&self) -> &str {
        "Bitget"
    }*/

    /*pub async fn get_ticker_info(&self, symbol: &str) -> Result<BitgetTickerInfo> {
        let url = format!(
            "https://api.bitget.com/api/v2/market/ticker?symbol={}",
            symbol
        );
        let req = reqwest::Client::new()
            .get(&url)
            .timeout(Duration::from_secs(10))
            .build()?;

        let ticker: BitgetTickerInfo = self
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
