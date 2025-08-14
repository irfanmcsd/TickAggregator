use crate::pkg::exchanges::binance::rate_limited_client::RateLimitedClient;
use crate::pkg::exchanges::exchange_entities::BinanceTickerInfo;
use crate::pkg::exchanges::exchange_entities::TickerInfo;
use crate::pkg::exchanges::exchange::ExchangeApi;
use anyhow::{Result, anyhow};
use async_trait::async_trait;

pub struct BinanceApi {
    client: RateLimitedClient,
}

impl BinanceApi {
    pub fn new() -> Self {
        Self {
            client: RateLimitedClient::new(None),
        }
    }
}

#[async_trait]
impl ExchangeApi for BinanceApi {
    async fn get_all_tickers(&self) -> Result<Vec<TickerInfo>> {
        let url = "https://fapi.binance.com/fapi/v1/ticker/24hr";
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

        let binance_tickers: Vec<BinanceTickerInfo> = serde_json::from_slice(&bytes)?;

        let standard_tickers = binance_tickers
            .into_iter()
            .map(|b| TickerInfo {
                symbol: b.symbol,
                last_price: b.last_price,
                vol_24h: Some(b.volume),
            })
            .collect();

        Ok(standard_tickers)
    }


    /*fn name(&self) -> &str {
        "Binance"
    }*/

    /*  pub async fn get_ticker_info(&self, symbol: &str) -> Result<TickerInfo> {
        let url = format!(
            "https://fapi.binance.com/fapi/v1/ticker/24hr?symbol={}",
            symbol
        );
        let req = reqwest::Client::new()
            .get(&url)
            .timeout(Duration::from_secs(10))
            .build()?;

        let ticker: TickerInfo = self
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
    }  */
}
