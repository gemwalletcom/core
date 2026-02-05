# Device WebSocket Streaming

## Overview

Real-time price, balance, and transaction updates via authenticated WebSocket connection using device authentication.

## Endpoint

```
wss://api.gemwallet.com/v2/devices/stream
```

## Authentication

Uses the same device authentication as all `/v2/devices/*` endpoints.

**For complete authentication details, see:** [Device Authentication](DEVICE_AUTHENTICATION.md)

## Protocol

### Client → Server Messages

**Subscribe to Prices:**
```json
{
  "type": "subscribePrices",
  "data": {
    "assets": ["bitcoin", "ethereum"]
  }
}
```

**Add More Assets:**
```json
{
  "type": "addPrices",
  "data": {
    "assets": ["solana"]
  }
}
```

**Unsubscribe from Prices:**
```json
{
  "type": "unsubscribePrices",
  "data": {
    "assets": ["bitcoin"]
  }
}
```

### Server → Client Messages

**Price Update:**
```json
{
  "event": "prices",
  "data": {
    "prices": [
      {
        "assetId": "bitcoin",
        "price": 45000.50,
        "priceChangePercentage24h": 2.5,
        "updatedAt": "2024-01-23T12:00:00Z"
      }
    ],
    "rates": [
      {
        "symbol": "USD",
        "rate": 1.0
      }
    ]
  }
}
```

**Balance Update:**
```json
{
  "event": "balances",
  "data": [
    {
      "walletId": "multicoin_0x742d35...",
      "assetId": "ethereum"
    }
  ]
}
```

**Transactions Update:**
```json
{
  "event": "transactions",
  "data": {
    "walletId": "multicoin_0x742d35...",
    "transactions": ["0xabc123...", "0xdef456..."]
  }
}
```

## Notes

- Authentication happens once during WebSocket upgrade
- Price updates are batched every 5 seconds
- Run as separate service: `api websocket_stream`

## Implementation

- Stream handler: [`apps/api/src/websocket_stream/stream.rs`](../apps/api/src/websocket_stream/stream.rs)
- Client logic: [`apps/api/src/websocket_stream/client.rs`](../apps/api/src/websocket_stream/client.rs)
- Message types: [`crates/primitives/src/stream.rs`](../crates/primitives/src/stream.rs)
- Price payload: [`crates/primitives/src/websocket.rs`](../crates/primitives/src/websocket.rs)
