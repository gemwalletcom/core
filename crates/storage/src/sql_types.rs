use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, Output, ToSql};
use primitives::rewards::RewardStatus as PrimitiveRewardStatus;
use std::io::Write;
use std::ops::Deref;
use std::str::FromStr;

use crate::schema::sql_types::RewardStatus as RewardStatusSql;

#[derive(Debug, Clone, Copy, PartialEq, AsExpression, FromSqlRow)]
#[diesel(sql_type = RewardStatusSql)]
pub struct RewardStatus(pub PrimitiveRewardStatus);

impl RewardStatus {
    pub const UNVERIFIED: Self = Self(PrimitiveRewardStatus::Unverified);
    pub const PENDING: Self = Self(PrimitiveRewardStatus::Pending);
    pub const VERIFIED: Self = Self(PrimitiveRewardStatus::Verified);
    pub const TRUSTED: Self = Self(PrimitiveRewardStatus::Trusted);
    pub const DISABLED: Self = Self(PrimitiveRewardStatus::Disabled);
}

impl Deref for RewardStatus {
    type Target = PrimitiveRewardStatus;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<PrimitiveRewardStatus> for RewardStatus {
    fn from(v: PrimitiveRewardStatus) -> Self {
        Self(v)
    }
}

impl From<RewardStatus> for PrimitiveRewardStatus {
    fn from(w: RewardStatus) -> Self {
        w.0
    }
}

impl FromSql<RewardStatusSql, Pg> for RewardStatus {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        let s = std::str::from_utf8(bytes.as_bytes())?;
        Ok(Self(PrimitiveRewardStatus::from_str(s).map_err(|e| format!("Invalid RewardStatus: {}", e))?))
    }
}

impl ToSql<RewardStatusSql, Pg> for RewardStatus {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        out.write_all(self.0.as_ref().as_bytes())?;
        Ok(serialize::IsNull::No)
    }
}
