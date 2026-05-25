# Wallet Authentication

## Overview

Wallet authentication endpoints require proof of wallet ownership via blockchain-native signatures (ECDSA for Ethereum). Used for referral/rewards operations and other authenticated wallet actions.

## Authentication Flow

1. Client requests nonce from `/v2/devices/auth/nonce`
2. Client signs `AuthMessage` with wallet private key
3. Client sends authenticated request with signature
4. Server processes request

## Authentication Request Structure

**Request Body:**
```json
{
  "auth": {
    "deviceId": "abc123-device-id",
    "chain": "ethereum",
    "address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
    "nonce": "550e8400-e29b-41d4-a716-446655440000",
    "signature": "0x1234567890abcdef..."
  },
  "data": {
    // Endpoint-specific payload
  }
}
```

Wallet-authenticated requests are still device-authenticated requests. The request body hash is included in the signed `Authorization: Gem ...` device-auth payload, which binds the wallet-signed JSON body to the request.

## Nonce Request

**Endpoint:**
```
GET /v2/devices/auth/nonce
```

**Response:**
```json
{
  "nonce": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": 1706000000
}
```

## Signature Generation

**AuthMessage Structure:**
```json
{
  "chain": "ethereum",
  "address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
  "authNonce": {
    "nonce": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": 1706000000
  }
}
```

**Signing Process:**
1. Serialize `AuthMessage` to JSON string
2. Compute Keccak256 hash (for Ethereum)
3. Sign hash with wallet private key (ECDSA)
4. Encode as hex with `0x` prefix

## Request Example

```
POST https://api.gemwallet.com/v2/devices/rewards/referrals/create
Content-Type: application/json
Authorization: Gem base64(<device_id_hex>.<timestamp_ms>.<wallet_id>.<body_hash_hex>.<signature_hex>)

{
  "auth": {
    "deviceId": "abc123-device-id",
    "chain": "ethereum",
    "address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
    "nonce": "550e8400-e29b-41d4-a716-446655440000",
    "signature": "0xf8e7d6c5b4a3..."
  },
  "data": {
    "code": "myusername"
  }
}
```

## Implementation

References for implementation details:
- Wallet signature verification: `crates/gem_auth/src/signature.rs`
- Authentication guards: `apps/api/src/auth/guard.rs`
- Nonce management: `crates/gem_auth/src/client.rs`
- Auth primitives: `crates/primitives/src/auth.rs`
- Tests: `crates/gem_auth/src/signature.rs#L48`
