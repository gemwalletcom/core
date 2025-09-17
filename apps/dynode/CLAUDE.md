# Dynode

A high-performance blockchain proxy service written in Rust that routes requests to multiple blockchain nodes with automatic failover and load balancing.

## What it does

- **Multi-chain proxy**: Supports 9 blockchains (Ethereum, Bitcoin, Solana, Cosmos, Ton, Tron, Aptos, Sui, XRP, Near)
- **Intelligent routing**: Monitors node health and automatically switches to the best performing node
- **Metrics collection**: Exposes Prometheus metrics for monitoring proxy performance and node status
- **Block synchronization**: Tracks latest block numbers to ensure nodes are synchronized

## Architecture

- `main.rs` - Starts dual HTTP servers (proxy + metrics)
- `node_service.rs` - Manages node health monitoring and failover logic
- `proxy_request_service.rs` - Handles incoming requests and proxies to blockchain nodes  
- `chain_service/` - Blockchain-specific API implementations
- `metrics.rs` - Prometheus metrics collection
- `config.rs` - YAML configuration management

## Configuration

Uses `config.yml` to define:
- Blockchain domains and their RPC endpoints
- Node health check intervals and block delay thresholds
- Custom headers and URL overrides per endpoint
- User agent pattern matching for metrics

## Commands

```bash
cargo build      # Build the project
cargo run        # Start the proxy service  
cargo test       # Run tests
cargo clippy     # Code quality checks
```

## Metrics

Exposes metrics on `/metrics` endpoint including:
- Request counts by host and user agent
- Response latency histograms  
- Current active node per domain
- Block height tracking