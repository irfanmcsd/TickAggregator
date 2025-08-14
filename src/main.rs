mod pkg;

use crate::pkg::aggregator::{symbol_rotator::SymbolRotator, ticker_aggregator::KlineAggregator};
use crate::pkg::config::SETTINGS;
use crate::pkg::db::DB;
use crate::pkg::dbcontext::kline::save_klines;
use crate::pkg::exchanges::exchange_client::core_futures_all_tickers;
use crate::pkg::exchanges::exchange_entities::TickerInfo;

use dotenv::dotenv;
use env_logger::Env;
use log::{error, info, warn};
use rand::Rng;
use std::{collections::HashSet, sync::Arc, time::Duration};
use tokio::signal;
use tokio::time::{Instant, interval_at};

#[tokio::main]
async fn main() {
    dotenv().ok();

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("📈 App started");

    let settings_ref = Arc::clone(&SETTINGS);

    // Initialize DB connection
    let db = match DB::init(&settings_ref.database).await {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to connect to DB: {:?}", e);
            return;
        }
    };
    info!("🗃️ Database initialized");

    // Auto-migrate schema
    if let Err(e) = pkg::dbcontext::migration::auto_migrate(&db.pool).await {
        error!("❌ AutoMigrate failed: {:?}", e);
        return;
    }
    info!("✅ Auto-migration complete");

    let exchange = settings_ref.exchange.to_lowercase();
    let symbols = settings_ref.symbols.clone();

    let mut invalid_symbols: HashSet<String> = settings_ref
        .blacklisted_symbols
        .iter()
        .map(|s| s.to_uppercase())
        .collect();

    let batch_size = 50;
    let mut rotator = SymbolRotator::new(symbols, batch_size);

    let k_agg = KlineAggregator::new(settings_ref.debug);

    let refresh_interval_secs = if settings_ref.refresh_seconds < 4 {
        4
    } else {
        settings_ref.refresh_seconds
    };
    let interval_duration = Duration::from_secs(refresh_interval_secs as u64);

    let mut ticker = interval_at(Instant::now() + interval_duration, interval_duration);

    info!("⏳ Starting periodic fetch loop for exchange: {}", exchange);

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                info!("🔄 New fetch cycle started");

                // Random jitter
                let jitter = rand::thread_rng().gen_range(0..500);
                tokio::time::sleep(Duration::from_millis(jitter)).await;

                // Get next batch
                let raw_batch = rotator.next_batch();
                let batch: Vec<String> = raw_batch
                    .into_iter()
                    .flat_map(|slice| slice.iter())
                    .filter(|sym| !invalid_symbols.contains(&sym.to_uppercase()))
                    .cloned()
                    .collect();

                info!("📦 Processing batch of {} symbols", batch.len());

                if batch.is_empty() {
                    warn!("⚠️ No valid symbols in batch to process");
                    continue;
                }

                // Fetch tickers
                info!("🌐 Fetching tickers from {}", exchange);
                let tickers_res = core_futures_all_tickers(&exchange).await;
                let tickers = match tickers_res {
                    Ok(data) => {
                        info!("✅ Retrieved {} total tickers from {}", data.len(), exchange);
                        data
                    }
                    Err(e) => {
                        error!("❌ Failed to fetch tickers: {:?}", e);
                        continue;
                    }
                };

                // Filter tickers for batch
                let batch_set: HashSet<String> = batch.iter().map(|s| s.to_uppercase()).collect();
                let filtered: Vec<TickerInfo> = tickers.into_iter()
                    .filter(|t| batch_set.contains(&t.symbol.to_uppercase()))
                    .collect();

                info!("📥 Matched {} tickers from batch request", filtered.len());

                // Blacklist missing symbols
                let found_symbols: HashSet<String> = filtered.iter()
                    .map(|t| t.symbol.to_uppercase())
                    .collect();

                for sym in &batch {
                    let up = sym.to_uppercase();
                    if !found_symbols.contains(&up) && !invalid_symbols.contains(&up) {
                        warn!("🚫 Symbol {} not found, blacklisting", up);
                        invalid_symbols.insert(up.clone());

                        if let Err(e) = pkg::save_config::save_config("appsettings.yaml").await {
                            error!("❌ Failed to save config: {:?}", e);
                        }
                    }
                }

                // Feed aggregator
                for t in &filtered {
                    match (t.last_price.parse::<f64>(), t.vol_24h.as_deref()) {
                        (Ok(price), Some(vol_str)) => {
                            if let Ok(volume) = vol_str.parse::<f64>() {
                                k_agg.add_price(&t.symbol, price, volume).await;
                            } else {
                                warn!("❌ Failed to parse volume for symbol {}", t.symbol);
                            }
                        }
                        _ => warn!("❌ Failed to parse price or volume for symbol {}", t.symbol),
                    }
                }
                info!("📊 Aggregator updated with {} tickers", filtered.len());

                // Flush intervals
                let flush_intervals = get_flush_intervals();
                if k_agg.debug {
                    info!("[Debug] Flushing intervals: {:?}", flush_intervals);
                }

                if !flush_intervals.is_empty() {
                    let flush_refs: Vec<&str> = flush_intervals.iter().map(|s| s.as_str()).collect();
                    let kline_data = k_agg.extract_ohlc(&flush_refs).await;

                    if !kline_data.is_empty() {
                        info!("📝 Preparing to save {} OHLC records", kline_data.len());
                        if let Err(e) = save_klines(&db.pool, &kline_data, &SETTINGS.instance).await {
                            error!("❌ Failed to save klines: {:?}", e);
                        } else {
                            info!("💾 Successfully saved {} OHLC entries to DB", kline_data.len());
                        }
                    } else {
                        info!("ℹ️ No OHLC data to save this cycle");
                    }
                }
            },
            _ = signal::ctrl_c() => {
                info!("🛑 Shutdown signal received");
                break;
            }
        }
    }

    info!("👋 App shutdown complete");
}
fn get_flush_intervals() -> Vec<String> {
    // For now, always flush 1m interval klines
    vec!["1m".to_string()]
}
