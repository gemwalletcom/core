use primitives::{Asset, AssetId, AssetType, Chain};

pub type GemAsset = Asset;
pub type GemAssetId = AssetId;
pub type GemAssetType = AssetType;

#[allow(non_camel_case_types)]
#[uniffi::remote(Enum)]
pub enum GemAssetType {
    NATIVE,
    ERC20,
    BEP20,
    SPL,
    SPL2022,
    TRC20,
    TOKEN,
    IBC,
    JETTON,
    SYNTH,
    ASA,
    PERPETUAL,
}

#[uniffi::remote(Record)]
pub struct GemAsset {
    pub id: GemAssetId,
    pub chain: Chain,
    pub token_id: Option<String>,
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
    pub asset_type: GemAssetType,
}

pub fn get_default_rank(chain: Chain) -> i32 {
    chain.rank()
}

pub fn get_asset(chain: Chain) -> GemAsset {
    Asset::from_chain(chain)
}

#[uniffi::export]
pub fn asset_default_rank(chain: Chain) -> i32 {
    get_default_rank(chain)
}

#[uniffi::export]
pub fn asset_wrapper(chain: Chain) -> GemAsset {
    get_asset(chain)
}
