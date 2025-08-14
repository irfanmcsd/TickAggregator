use crate::pkg::dbcontext::entities::{SymbolKlineData, clean_symbol};
use anyhow::Result;
use log::info;
use sqlx::{PgPool, Postgres, Transaction, postgres::PgQueryResult};
use sqlx::Executor;

pub async fn save_klines(pool: &PgPool, data: &[SymbolKlineData], instance: &str) -> Result<()> {
    if data.is_empty() {
        info!("📭 No klines to insert for instance {}", instance);
        return Ok(());
    }

    // Precompute cleaned + prepared parameters
    let params: Vec<_> = data
        .iter()
        .map(|kline| {
            (
                clean_symbol(&kline.symbol),
                kline.interval.clone(),
                kline.open,
                kline.high,
                kline.low,
                kline.close,
                kline.open_time,
                instance.to_string(), // moved outside loop chunk processing
                kline.volume,
                kline.trade_count,
            )
        })
        .collect();

    let batch_size = 100;
    let mut tx: Transaction<'_, Postgres> = pool.begin().await?;

    for chunk in params.chunks(batch_size) {
        let mut query_builder = sqlx::QueryBuilder::<Postgres>::new(
            "INSERT INTO \"Dev_SymbolKlineData\" \
        (symbol, interval, open, high, low, close, open_time, instance, volume, trade_count) ",
        );

        query_builder.push_values(
            chunk,
            |mut b,
             (
                symbol,
                interval,
                open,
                high,
                low,
                close,
                open_time,
                instance,
                volume,
                trade_count,
            )| {
                b.push_bind(symbol)
                    .push_bind(interval)
                    .push_bind(open)
                    .push_bind(high)
                    .push_bind(low)
                    .push_bind(close)
                    .push_bind(open_time)
                    .push_bind(instance)
                    .push_bind(volume)
                    .push_bind(trade_count);
            },
        );

        query_builder.push(" ON CONFLICT (symbol, interval, open_time) DO NOTHING");

        // Borrow tx only for this statement
        {
            let query = query_builder.build();
            let result: PgQueryResult = query.execute(&mut *tx).await?;
            info!(
                "✅ Saved {} klines for instance {} (attempted {})",
                result.rows_affected(),
                instance,
                chunk.len()
            );
        }
    }

    tx.commit().await?;
    Ok(())
}
