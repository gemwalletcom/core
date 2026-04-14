# Rewards and Referrals

## Reward Activation

New users start in an unverified state. A background worker (`RewardsEligibilityChecker`) periodically evaluates pending users and promotes them to verified status once they meet the required activity thresholds (configurable via `RewardsEligibilityActiveDuration` and `RewardsEligibilityTransactionsCount`).

A `verify_after` timestamp may be set on the rewards record to enforce a minimum waiting period before activation can occur, regardless of activity.

## Referral Flow

1. **User1** (verified) shares their referral code
2. **User2** redeems the code via `use_referral_code`
3. A pending referral is created and a `verify_after` delay is set on User2's record (configured via `ReferralVerificationDelay`)
4. User2 must return after the delay period expires to complete activation
5. On activation, the referral is finalized and both users receive reward events

## Key Config

| Key | Purpose |
|-----|---------|
| `RewardsEligibilityActiveDuration` | Minimum activity duration for organic promotion |
| `RewardsEligibilityTransactionsCount` | Minimum confirmed transactions for promotion |
| `RewardsTimerEligibilityChecker` | Interval for the background eligibility worker |
| `ReferralVerificationDelay` | Waiting period before a referred user can be activated |
