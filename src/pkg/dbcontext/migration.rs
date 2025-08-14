use sqlx::PgPool;

pub async fn auto_migrate(pool: &PgPool) -> Result<(), sqlx::Error> {
    let query = r#"
    CREATE TABLE IF NOT EXISTS "Dev_SymbolKlineData" (
        id SERIAL PRIMARY KEY,
        symbol VARCHAR(50) NOT NULL,
        interval VARCHAR(10) NOT NULL DEFAULT '1m',
        open NUMERIC(18,8) NOT NULL,
        high NUMERIC(18,8) NOT NULL,
        low NUMERIC(18,8) NOT NULL,
        close NUMERIC(18,8) NOT NULL,
        open_time BIGINT NOT NULL,
        instance VARCHAR(255),
        volume DOUBLE PRECISION NOT NULL,
        trade_count BIGINT NOT NULL,
        CONSTRAINT idx_symbol_interval_time UNIQUE(symbol, interval, open_time)
    );
    "#;

    sqlx::query(query).execute(pool).await?;
    Ok(())
}