use sqlx::PgPool;

pub async fn auto_migrate(pool: &PgPool) -> Result<(), sqlx::Error> {
    // 1️⃣ Create the table if it does not exist
    let create_table = r#"
    CREATE TABLE IF NOT EXISTS "Dev_SymbolKlineData" (
        id BIGSERIAL,
        symbol TEXT NOT NULL,
        interval TEXT NOT NULL DEFAULT '1m',
        open NUMERIC(18,8) NOT NULL,
        high NUMERIC(18,8) NOT NULL,
        low NUMERIC(18,8) NOT NULL,
        close NUMERIC(18,8) NOT NULL,
        open_time TIMESTAMPTZ NOT NULL,
        instance TEXT,
        volume DOUBLE PRECISION NOT NULL,
        trade_count BIGINT NOT NULL
    );
    "#;
    sqlx::query(create_table).execute(pool).await?;

    // 2️⃣ Drop old primary key or unique constraint if exists
    let drop_pk = r#"
    DO $$
    BEGIN
        IF EXISTS (
            SELECT 1
            FROM information_schema.table_constraints
            WHERE table_name='Dev_SymbolKlineData'
            AND constraint_type='PRIMARY KEY'
        ) THEN
            ALTER TABLE "Dev_SymbolKlineData" DROP CONSTRAINT "Dev_SymbolKlineData_pkey";
        END IF;
        
        IF EXISTS (
            SELECT 1
            FROM information_schema.table_constraints
            WHERE table_name='Dev_SymbolKlineData'
            AND constraint_type='UNIQUE'
            AND constraint_name='idx_symbol_interval_time'
        ) THEN
            ALTER TABLE "Dev_SymbolKlineData" DROP CONSTRAINT idx_symbol_interval_time;
        END IF;
    END$$;
    "#;
    sqlx::query(drop_pk).execute(pool).await?;

    // 3️⃣ Add composite primary key (symbol, interval, open_time)
    let add_pk = r#"
    ALTER TABLE "Dev_SymbolKlineData"
    ADD CONSTRAINT idx_symbol_interval_time PRIMARY KEY (symbol, interval, open_time);
    "#;
    sqlx::query(add_pk).execute(pool).await?;

    // 4️⃣ Convert table into a hypertable (if not already)
    let create_hypertable = r#"
    DO $$
    BEGIN
        IF NOT EXISTS (
            SELECT 1 FROM timescaledb_information.hypertables
            WHERE hypertable_name = 'Dev_SymbolKlineData'
        ) THEN
            PERFORM create_hypertable(
                '"Dev_SymbolKlineData"',
                'open_time',
                chunk_time_interval => INTERVAL '1 day',
                migrate_data => true
            );
        END IF;
    END$$;
    "#;
    sqlx::query(create_hypertable).execute(pool).await?;

    // 5️⃣ Enable compression for historical chunks
    let enable_compression = r#"
    ALTER TABLE "Dev_SymbolKlineData"
    SET (
        timescaledb.compress,
        timescaledb.compress_segmentby = 'symbol, interval'
    );
    "#;
    sqlx::query(enable_compression).execute(pool).await?;

    Ok(())
}