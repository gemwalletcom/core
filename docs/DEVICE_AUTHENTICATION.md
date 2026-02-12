# Device Authentication

## Overview

All `/v2/devices/*` endpoints require authentication via Ed25519 request signing. The server supports two authentication methods: **Gem** (recommended) and **individual headers** (legacy).

## Method 1: Gem Authorization Header (Recommended)

```
Authorization: Gem base64(<device_id_hex>.<timestamp_ms>.<wallet_id>.<body_hash_hex>.<signature_hex>)
```

The payload is always 5 dot-separated parts. After base64-decoding:
- `device_id_hex` - 64-character hex Ed25519 public key
- `timestamp_ms` - Unix timestamp in milliseconds
- `wallet_id` - Wallet identifier (empty string for non-wallet endpoints)
- `body_hash_hex` - 64-character hex SHA256 hash of request body
- `signature_hex` - 128-character hex Ed25519 signature

When `wallet_id` is empty, the payload contains `..` (two consecutive dots).

**Signed message:**
```
{timestamp}.{method}.{path}.{walletId}.{bodyHash}
```

`walletId` is empty for non-wallet endpoints, producing `..` in the message:

```
1706000000000.GET./v2/devices..e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
1706000000000.GET./v2/devices/assets.multicoin_0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb.e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
```

## Method 2: Individual Headers (Legacy)

**Required:**
- `x-device-id`: Unique device identifier (64-character hex Ed25519 public key)
- `x-device-signature`: Ed25519 signature (hex or base64)
- `x-device-timestamp`: Unix timestamp in milliseconds
- `x-device-body-hash`: SHA256 hash of request body (hex)

**Optional (wallet-specific endpoints):**
- `x-wallet-id`: Wallet identifier (format: `multicoin_{address}`)

**Signed message:**
```
v1.{timestamp}.{method}.{path}.{bodyHash}
```

Note: `walletId` is **not** included in the legacy signed message.

## Request Examples

### Gem (wallet-scoped)
```http
GET /v2/devices/assets?from_timestamp=1234567890
Authorization: Gem base64(abc123...def456.1706000000000.multicoin_0x742d...f0bEb.e3b0c44...b855.aabb11...)
```

### Gem (no wallet)
```http
GET /v2/devices
Authorization: Gem base64(abc123...def456.1706000000000..e3b0c44...b855.aabb11...)
```

### Individual Headers (legacy)
```http
GET /v2/devices/assets?from_timestamp=1234567890
x-device-id: abc123...def456
x-wallet-id: multicoin_0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb
x-device-signature: aabb11...
x-device-timestamp: 1706000000000
x-device-body-hash: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
```

## Implementation

- Request signature verification: [`apps/api/src/devices/signature.rs`](../apps/api/src/devices/signature.rs)
- Cryptographic verification: [`crates/gem_auth/src/device_signature.rs`](../crates/gem_auth/src/device_signature.rs)
- Request guards: [`apps/api/src/devices/guard.rs`](../apps/api/src/devices/guard.rs)
- Error handling: [`apps/api/src/devices/error.rs`](../apps/api/src/devices/error.rs)
