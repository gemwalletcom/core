use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, Output, ToSql};
use primitives::nft::NFTType as PrimitiveNFTType;
use primitives::rewards::{
    RedemptionStatus as PrimitiveRedemptionStatus, RewardEventType as PrimitiveRewardEventType,
    RewardRedemptionType as PrimitiveRewardRedemptionType, RewardStatus as PrimitiveRewardStatus,
};
use primitives::scan::AddressType as PrimitiveAddressType;
use primitives::{
    AssetType as PrimitiveAssetType, LinkType as PrimitiveLinkType, TransactionType as PrimitiveTransactionType,
};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::ops::Deref;
use std::str::FromStr;

use crate::schema::sql_types::{
    AddressType as AddressTypeSql, AssetType as AssetTypeSql, LinkType as LinkTypeSql, NftType as NftTypeSql,
    RedemptionStatus as RedemptionStatusSql, RewardEventType as RewardEventTypeSql,
    RewardRedemptionType as RewardRedemptionTypeSql, RewardStatus as RewardStatusSql, TransactionType as TransactionTypeSql,
};

macro_rules! diesel_enum {
    ($wrapper:ident, $inner:ty, $sql_type:ty, [$($variant:ident),+ $(,)?]) => {
        #[derive(Debug, Clone, Serialize, Deserialize, AsExpression, FromSqlRow)]
        #[serde(transparent)]
        #[diesel(sql_type = $sql_type)]
        pub struct $wrapper(pub $inner);

        #[allow(non_upper_case_globals)]
        impl $wrapper {
            $(pub const $variant: Self = Self(<$inner>::$variant);)+
        }

        impl Deref for $wrapper {
            type Target = $inner;
            fn deref(&self) -> &Self::Target { &self.0 }
        }

        impl From<$inner> for $wrapper {
            fn from(v: $inner) -> Self { Self(v) }
        }

        impl From<$wrapper> for $inner {
            fn from(w: $wrapper) -> Self { w.0 }
        }

        impl FromSql<$sql_type, Pg> for $wrapper {
            fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
                let s = std::str::from_utf8(bytes.as_bytes())?;
                Ok(Self(<$inner>::from_str(s).map_err(|e| format!("Invalid {}: {}", stringify!($wrapper), e))?))
            }
        }

        impl ToSql<$sql_type, Pg> for $wrapper {
            fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
                out.write_all(self.0.as_ref().as_bytes())?;
                Ok(serialize::IsNull::No)
            }
        }
    };
}

diesel_enum!(RewardStatus, PrimitiveRewardStatus, RewardStatusSql, [
    Unverified, Pending, Verified, Trusted, Disabled
]);

diesel_enum!(RewardRedemptionType, PrimitiveRewardRedemptionType, RewardRedemptionTypeSql, [
    Asset, GiftAsset
]);

diesel_enum!(RedemptionStatus, PrimitiveRedemptionStatus, RedemptionStatusSql, [
    Pending, Completed, Failed
]);

diesel_enum!(TransactionType, PrimitiveTransactionType, TransactionTypeSql, [
    Transfer, TransferNFT, Swap, TokenApproval, StakeDelegate, StakeUndelegate, StakeRewards,
    StakeRedelegate, StakeWithdraw, StakeFreeze, StakeUnfreeze, AssetActivation, SmartContractCall,
    PerpetualOpenPosition, PerpetualClosePosition, PerpetualModifyPosition
]);

diesel_enum!(LinkType, PrimitiveLinkType, LinkTypeSql, [
    X, Discord, Reddit, Telegram, GitHub, YouTube, Facebook, Website, Coingecko, OpenSea,
    Instagram, MagicEden, CoinMarketCap, TikTok
]);

diesel_enum!(NftType, PrimitiveNFTType, NftTypeSql, [
    ERC721, ERC1155, SPL, JETTON
]);

diesel_enum!(AssetType, PrimitiveAssetType, AssetTypeSql, [
    NATIVE, ERC20, BEP20, SPL, SPL2022, TRC20, TOKEN, IBC, JETTON, SYNTH, ASA, PERPETUAL, SPOT
]);

diesel_enum!(AddressType, PrimitiveAddressType, AddressTypeSql, [
    Address, Contract, Validator
]);

diesel_enum!(RewardEventType, PrimitiveRewardEventType, RewardEventTypeSql, [
    CreateUsername, InvitePending, InviteNew, InviteExisting, Joined, Disabled
]);
