# KlineCollector

A high-performance Kline (OHLC) data aggregator for multiple crypto exchanges.  
Collects 1-minute (configurable) candlestick data for hundreds of symbols, stores it in PostgreSQL, and provides structured datasets for strategy simulation and AI model training.

---

## Features

- **Multi-exchange support**: Binance, OKX, Bybit, Bitget (easily extendable).  
- **Configurable intervals**: Default 1-minute, adjustable via config.  
- **High concurrency**: Aggregates hundreds of symbols concurrently.  
- **Persistent storage**: Saves OHLC data in PostgreSQL with conflict handling.  
- **AI/strategy ready**: Structured dataset output for ML model training or backtesting.  
- **Streaming support**: Optional integration with Redis/Kafka for live tick streaming.  
- **Robust error handling**: Retry mechanisms for exchange APIs.  

---

## Dependencies

- **Rust** (latest stable)
- **Tokio** — async runtime
- **SQLx** — PostgreSQL async client
- **Serde** — config serialization/deserialization
- **Reqwest** — HTTP client
- **Anyhow** — error handling
- **OnceCell** — global lazy initialization

## License

MIT License. See [LICENSE](LICENSE) for details.