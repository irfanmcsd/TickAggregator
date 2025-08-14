use anyhow::{Result, anyhow};
use crate::pkg::exchanges::exchange_entities::TickerInfo;
use crate::pkg::exchanges::exchange::ExchangeApi;
use crate::pkg::exchanges::binance::binance_api::BinanceApi;
use crate::pkg::exchanges::bitget::{self};
use crate::pkg::exchanges::bybit::{self};
use crate::pkg::exchanges::okx::{self};


pub async fn core_futures_all_tickers(exchange: &str) -> Result<Vec<TickerInfo>> {
    let exchange = exchange.to_lowercase();

    let api: Box<dyn ExchangeApi> = match exchange.as_str() {
        "binance" => Box::new(BinanceApi::new()),
        "okx"     => Box::new(okx::okx_api::OkxApi::new()),
        "bitget"  => Box::new(bitget::bitget_api::BitgetApi::new()),
        "bybit"   => Box::new(bybit::bybit_api::BybitApi::new()),
        _ => return Err(anyhow!("unsupported exchange: {}", exchange)),
    };

    api.get_all_tickers().await
}