use async_trait::async_trait;
use anyhow::Result;
use crate::pkg::exchanges::exchange_entities::TickerInfo;

#[async_trait]
pub trait ExchangeApi: Send + Sync {
    async fn get_all_tickers(&self) -> Result<Vec<TickerInfo>>;
    fn name(&self) -> &str;
}