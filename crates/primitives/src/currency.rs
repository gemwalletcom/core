use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Debug, Clone, PartialEq, Eq, Hash, AsRefStr, EnumString, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[strum(serialize_all = "UPPERCASE")]
pub enum Currency {
    MXN,
    CHF,
    CNY,
    THB,
    HUF,
    AUD,
    IDR,
    RUB,
    ZAR,
    EUR,
    NZD,
    SAR,
    SGD,
    BMD,
    KWD,
    HKD,
    JPY,
    GBP,
    DKK,
    KRW,
    PHP,
    CLP,
    TWD,
    PKR,
    BRL,
    CAD,
    BHD,
    MMK,
    VEF,
    VND,
    CZK,
    TRY,
    INR,
    ARS,
    BDT,
    NOK,
    USD,
    LKR,
    ILS,
    PLN,
    NGN,
    UAH,
    XDR,
    MYR,
    AED,
    SEK,
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
