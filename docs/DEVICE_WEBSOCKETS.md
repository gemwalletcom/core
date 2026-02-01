# Device WebSocket Streaming

## Overview

Real-time price and balance updates via authenticated WebSocket connection using device authentication.

## Endpoint

```
wss://api.gemwallet.com/v2/devices/stream
```

## Authentication

Uses the same device authentication as all `/v2/devices/*` endpoints.

**Required HTTP Headers:**
- `x-device-id`: Unique device identifier
- `x-device-signature`: Base64-encoded Ed25519 signature
- `x-device-timestamp`: Unix timestamp in milliseconds
- `x-device-body-hash`: SHA256 hash of request body

**Authentication Flow:**
```
Client                            Server                          Database
  |                                 |                                 |
  |--- WebSocket Upgrade ---------> |                                 |
  |    (x-device-id,                |                                 |
  |     x-device-signature,         |                                 |
  |     x-device-timestamp,         |                                 |
  |     x-device-body-hash)         |                                 |
  |                                 |                                 |
  |                                 |--- Verify Signature ----------> |
  |                                 | <-- Device Exists ------------- |
  |                                 |                                 |
  | <-- 101 Switching Protocols --- |                                 |
  |                                 |                                 |
  |=== WebSocket Connected =======|
```

**For complete authentication details, see:** [Device Authentication Documentation](DEVICE_AUTHENTICATION.md)

## Protocol

### Client → Server Messages

**Subscribe to Prices:**
```json
{
  "type": "subscribe_prices",
  "assets": ["bitcoin", "ethereum"]
}
```

**Add More Assets:**
```json
{
  "type": "add_prices",
  "assets": ["solana"]
}
```

**Unsubscribe from Prices:**
```json
{
  "type": "unsubscribe_prices",
  "assets": ["bitcoin"]
}
```

**Subscribe to Balances (stub, not implemented):**
```json
{
  "type": "subscribe_balances"
}
```

**Unsubscribe from Balances (stub, not implemented):**
```json
{
  "type": "unsubscribe_balances"
}
```

### Server → Client Messages

**Price Update (every 5 seconds):**
```json
{
  "prices": [
    {
      "asset_id": "bitcoin",
      "price": 45000.50,
      "price_change_percentage_24h": 2.5
    }
  ],
  "rates": [
    {
      "code": "USD",
      "rate": 1.0
    }
  ]
}
```

**Balance Update (stub, not implemented):**
```json
{
  "wallet_id": "multicoin_0x742d35...",
  "chain": "ethereum",
  "address": "0x742d35...",
  "asset_id": "ethereum"
}
```

## Message Flow

```
Client                            Server                          Redis
  |                                 |                                |
  |=== WebSocket Connected =====|
  |                                 |                                |
  |--- Subscribe Message ---------> |                                |
  |    {"type": "subscribe_prices", |                                |
  |     "assets": ["bitcoin"]}      |                                |
  |                                 |                                |
  |                                 |--- SUBSCRIBE bitcoin --------> |
  |                                 |                                |
  | <-- Price Response ------------ |                                |
  |     {"prices": [...]}           |                                |
  |                                 |                                |
  |                                 | <-- PUBLISH price update ----- |
  |                                 |     (batched)                  |
  |                                 |                                |
  | <-- Price Update (5s batch) --- |                                |
  |     {"prices": [...]}           |                                |
```

## Message Types

### Prices

**`subscribe_prices`**
- Replaces all currently tracked assets
- Returns current prices immediately with fiat rates
- Updates batched every 5 seconds via Redis pub/sub

**`add_prices`**
- Adds assets to existing tracked list
- Returns current prices immediately without fiat rates

**`unsubscribe_prices`**
- Removes specific assets from tracking
- Returns updated price list

### Balances (stub, not implemented)

**`subscribe_balances`**
- Enable balance updates for all device addresses
- No asset filter - receives updates for all balances

**`unsubscribe_balances`**
- Disable balance updates

## Notes

- Authentication happens once during WebSocket upgrade (same as HTTP API)
- Connection requires valid device in database
- Price updates are batched every 5 seconds
- Balance messages are defined but not yet implemented
- Run as separate service: `api websocket_stream`
