# Rewards and Referrals

## Activation Flow

```
[New User] ─> create_username ─> [Unverified] ─> worker checks activity ─> [Verified] or [Trusted]
```

## Referral Flow

```
User1 (Verified): shares code
  ─> User2 redeems (same endpoint)
     ─> status = Pending, verify_after = now + delay
  ─> delay passes
  ─> User2 calls same endpoint again
     ─> status reset to Unverified, verify_after cleared
     ─> referral marked verified_at = now
     ─> both get reward events (InviteNew / Joined)
  ─> worker later promotes User2 Unverified ─> Verified
```

## Statuses

| Status | Can Invite | Description |
|--------|-----------|-------------|
| `Unverified` | No | Default after username creation. Awaiting promotion by worker. |
| `Pending` | No | Used a referral code, awaiting `verify_after` delay. |
| `Verified` | Yes | Promoted by worker. Can share referral code. |
| `Trusted` | Yes | Higher-tier verified. Higher referral limits. |
| `Disabled` | No | Account disabled. |

## Worker Promotion

`RewardsEligibilityChecker` promotes `Unverified` users to `Verified` when activity thresholds are met (`RewardsEligibilityActiveDuration`, `RewardsEligibilityTransactionsCount`). No explicit user action needed.

## Client UI States

| State | UI |
|-------|----|
| No username | "Get Started" button |
| `Unverified`, no `verify_after` | Rewards not active yet message |
| `verify_after` in future | "Bonus Pending" + countdown, confirm disabled |
| `verify_after` in past | "Your bonus is ready!", confirm enabled |
| `Verified`/`Trusted` | Invite Friends + share button |
| `Disabled` | Error with `disable_reason` |

## Key Config

| Key | Purpose |
|-----|---------|
| `RewardsEligibilityActiveDuration` | Min activity duration for promotion |
| `RewardsEligibilityTransactionsCount` | Min confirmed transactions for promotion |
| `RewardsTimerEligibilityChecker` | Worker check interval |
| `RewardsEligibilityPromotionLimit` | Max users promoted per worker run |
| `ReferralVerificationDelay` | Delay before referral confirmation |
