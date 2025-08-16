use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{TimeZone, Utc};
use log::debug;
use tokio::sync::Mutex;

use crate::pkg::dbcontext::entities::SymbolKlineData;

#[derive(Clone, Debug)]
pub struct TickData {
    pub price: f64,
    pub time: i64, // milliseconds since epoch
    pub volume: f64,
}

pub struct KlineAggregator {
    tick_buffer: Mutex<HashMap<String, HashMap<String, Vec<TickData>>>>,
    interval_to_ms: HashMap<String, i64>,
    max_interval_ms: i64,
    pub debug: bool,
}

impl KlineAggregator {
    pub fn new(debug: bool) -> Self {
        let intervals = HashMap::from([
            ("1m".to_string(), 60_000),
            // add more intervals here if needed
        ]);

        // Find max interval in milliseconds
        let max_interval_ms = intervals.values().copied().max().unwrap_or(60_000);

        Self {
            tick_buffer: Mutex::new(HashMap::new()),
            interval_to_ms: intervals,
            max_interval_ms,
            debug,
        }
    }

    /// Add a new price tick for a symbol
    pub async fn add_price(&self, symbol: &str, price: f64, volume: f64) {
        let mut buffer = self.tick_buffer.lock().await;

        let now = current_millis();
        let truncated = now - (now % 1000); // truncate to nearest second in ms

        let tick = TickData {
            price,
            time: truncated,
            volume,
        };

        // Ensure symbol entry exists
        let interval_map = buffer
            .entry(symbol.to_string())
            .or_insert_with(HashMap::new);

        for (interval, _) in &self.interval_to_ms {
            let ticks = interval_map
                .entry(interval.clone())
                .or_insert_with(Vec::new);

            // Only add if no tick with same timestamp exists
            if !ticks.iter().any(|t| t.time == truncated) {
                ticks.push(tick.clone());
                if self.debug {
                    debug!("Added tick to {} [{}]: {:.2}", symbol, interval, price);
                }
            }
        }
    }

    /// Extract OHLC data for requested intervals
    pub async fn extract_ohlc(&self, intervals: &[&str]) -> Vec<SymbolKlineData> {
        let mut buffer = self.tick_buffer.lock().await;
        let now = current_millis();
        let mut result = Vec::new();

        for (symbol, interval_map) in buffer.iter_mut() {
            for &interval in intervals {
                let ticks = match interval_map.get_mut(interval) {
                    Some(t) if !t.is_empty() => t,
                    _ => continue,
                };

                // Sort ticks by time ascending
                ticks.sort_unstable_by_key(|t| t.time);

                let interval_ms = match self.interval_to_ms.get(interval) {
                    Some(ms) => *ms,
                    None => continue,
                };

                let candle_end = now - (now % interval_ms);
                let candle_start = candle_end - interval_ms;

                // Binary search start and end indices
                let start_idx = ticks.partition_point(|t| t.time < candle_start);
                let end_idx = ticks.partition_point(|t| t.time < candle_end);

                if start_idx == end_idx {
                    continue;
                }

                let group = &ticks[start_idx..end_idx];
                let kline = Self::build_kline(symbol, interval, group, candle_start);
                result.push(kline);

                // Remove used ticks
                let remaining = ticks.split_off(end_idx);
                *ticks = remaining;
            }
            self.cleanup_old_ticks(symbol, interval_map, now).await;
        }
        result
    }

    /// Clean up ticks older than retention for a symbol
    async fn cleanup_old_ticks(
        &self,
        symbol: &str,
        interval_map: &mut HashMap<String, Vec<TickData>>,
        now: i64,
    ) {
        let retention = self.max_interval_ms * 3;
        let oldest_valid = now - retention;

        for (interval, ticks) in interval_map.iter_mut() {
            let original_len = ticks.len();
            ticks.retain(|t| t.time >= oldest_valid);
            if self.debug && ticks.len() != original_len {
                debug!(
                    "Cleaned {} old ticks from {} [{}]",
                    original_len - ticks.len(),
                    symbol,
                    interval
                );
            }
        }
    }

    /// Build OHLC candle from a slice of TickData (already sorted)
    fn build_kline(
        symbol: &str,
        interval: &str,
        group: &[TickData],
        open_time: i64,
    ) -> SymbolKlineData {
        let open = group.first().unwrap().price;
        let close = group.last().unwrap().price;
        let mut high = open;
        let mut low = open;
        let mut volume_sum = 0.0;

        for tick in group {
            if tick.price > high {
                high = tick.price;
            }
            if tick.price < low {
                low = tick.price;
            }
            volume_sum += tick.volume;
        }

        SymbolKlineData {
            symbol: symbol.to_string(),
            interval: interval.to_string(),
            open,
            close,
            high,
            low,
            open_time: Utc.timestamp_millis(open_time), // ✅ convert i64 -> DateTime<Utc>
            volume: volume_sum,
            trade_count: group.len() as i64,
            instance: None, // or Some("your_instance".to_string())
        }
    }
}

// Helper: current time in milliseconds since epoch
fn current_millis() -> i64 {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    dur.as_millis() as i64
}
