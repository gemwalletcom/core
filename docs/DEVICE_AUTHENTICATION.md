# Device Authentication

## Overview

All `/v2/devices/*` endpoints require authentication via HTTP headers and Ed25519 request signing.

## Authentication Headers

**Required:**
- `x-device-id`: Unique device identifier
- `x-device-signature`: Base64-encoded Ed25519 signature
- `x-device-timestamp`: Unix timestamp in milliseconds
- `x-device-body-hash`: SHA256 hash of request body (hex)

**Optional (wallet-specific endpoints):**
- `x-wallet-id`: Wallet identifier (format: `{type}_{chain}_{address}` or `multicoin_{address}`)

## Request Signing

**Message Format:**
```
v1.{timestamp}.{method}.{path}.{bodyHash}
```

**Example:**
```
v1.1706000000000.GET./v2/devices/assets.e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
```

Components:
- `v1` - Protocol version
- `1706000000000` - Unix timestamp (milliseconds)
- `GET` - HTTP method
- `/v2/devices/assets` - Request path
- `e3b0c44...` - SHA256 hash of request body (empty body hash shown)

## Request Example

```http
GET https://api.gemwallet.com/v2/devices/assets?from_timestamp=1234567890
x-device-id: abc123-device-id
x-wallet-id: multicoin_0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb
x-device-signature: Gj8K7h3m...
x-device-timestamp: 1706000000000
x-device-body-hash: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
```

## Error Response

```json
{
  "error": {
    "message": "Missing header: x-device-id"
  }
}
```

## Implementation

- Request signature verification: [`apps/api/src/devices/signature.rs`](../apps/api/src/devices/signature.rs)
- Cryptographic verification: [`crates/gem_auth/src/device_signature.rs`](../crates/gem_auth/src/device_signature.rs)
- Request guards: [`apps/api/src/devices/guard.rs`](../apps/api/src/devices/guard.rs)
- Error handling: [`apps/api/src/devices/error.rs`](../apps/api/src/devices/error.rs)
- Tests: [`crates/gem_auth/src/device_signature.rs#L25`](../crates/gem_auth/src/device_signature.rs#L25)
