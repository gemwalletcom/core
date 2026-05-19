use super::sui_error;
use crate::{
    SwapperError,
    mayan::{
        constants::{SUI_CCTP_CORE_STATE, SUI_CCTP_DENY_LIST, SUI_CCTP_TOKEN_STATE, SUI_MCTP_FEE_MANAGER_STATE, SUI_MCTP_STATE, SUI_WORMHOLE_STATE},
        model::MayanMctpQuote,
    },
};
use gem_sui::{
    SuiClient, is_sui_coin,
    models::CoinAsset,
    tx_builder::{ResolvedObjectInput, TransactionBuilderInput},
};
use serde::Deserialize;
use std::collections::HashMap;

pub(super) struct PrefetchedSuiData {
    pub(super) transaction: TransactionBuilderInput,
    pub(super) input_coins: Vec<CoinAsset>,
    pub(super) objects: HashMap<String, ResolvedObjectInput>,
    pub(super) mctp_package_id: String,
    pub(super) fee_manager_package_id: Option<String>,
    pub(super) mctp_input_contract: String,
    pub(super) from_token_verified_address: String,
    pub(super) mctp_verified_input_address: String,
    pub(super) mctp_input_treasury: String,
}

impl PrefetchedSuiData {
    pub(super) async fn prefetch(client: &SuiClient, sender: &str, route: &MayanMctpQuote, gas_budget: u64) -> Result<Self, SwapperError> {
        let mctp_input_contract = route.mctp_input_contract.clone().ok_or(SwapperError::InvalidRoute)?;
        let from_token_verified_address = route.from_token.verified_address.clone().ok_or(SwapperError::InvalidRoute)?;
        let mctp_verified_input_address = route.mctp_verified_input_address.clone().ok_or(SwapperError::InvalidRoute)?;
        let mctp_input_treasury = route.mctp_input_treasury.clone().ok_or(SwapperError::InvalidRoute)?;
        let transaction = TransactionBuilderInput::prefetch(client, sender, gas_budget).await.map_err(sui_error)?;
        let has_auction = route.has_auction == Some(true);
        let input_coins = if route.from_token.contract.as_str() != mctp_input_contract.as_str() || is_sui_coin(&mctp_input_contract) {
            Vec::new()
        } else {
            client.get_coin_assets_by_type(sender, &mctp_input_contract).await.map_err(sui_error)?
        };
        let mut object_ids = vec![
            SUI_MCTP_STATE.to_string(),
            SUI_CCTP_CORE_STATE.to_string(),
            SUI_CCTP_TOKEN_STATE.to_string(),
            SUI_CCTP_DENY_LIST.to_string(),
            SUI_WORMHOLE_STATE.to_string(),
            from_token_verified_address.clone(),
            mctp_verified_input_address.clone(),
            mctp_input_treasury.clone(),
        ];
        if has_auction {
            object_ids.push(SUI_MCTP_FEE_MANAGER_STATE.to_string());
        }
        let fetched_objects = ResolvedObjectInput::fetch_multiple(client, object_ids.clone()).await.map_err(sui_error)?;
        if fetched_objects.len() != object_ids.len() {
            return Err(SwapperError::transaction_error("Failed to prefetch all Mayan Sui objects"));
        }
        let objects = object_ids.into_iter().zip(fetched_objects).collect();
        let mctp_package_id = fetch_mayan_sui_package_id(client, SUI_MCTP_STATE).await?;
        let fee_manager_package_id = if has_auction {
            Some(fetch_mayan_sui_package_id(client, SUI_MCTP_FEE_MANAGER_STATE).await?)
        } else {
            None
        };

        Ok(Self {
            transaction,
            input_coins,
            objects,
            mctp_package_id,
            fee_manager_package_id,
            mctp_input_contract,
            from_token_verified_address,
            mctp_verified_input_address,
            mctp_input_treasury,
        })
    }
}

async fn fetch_mayan_sui_package_id(client: &SuiClient, state_object_id: &str) -> Result<String, SwapperError> {
    let state: MayanStateObject = client.get_object_json(state_object_id.to_string()).await.map_err(sui_error)?;
    Ok(state.latest_package_id())
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum MayanStateObject {
    Direct(MayanStateFields),
    Fields { fields: MayanStateFields },
    Content { content: MayanStateContent },
    Data { data: MayanStateContent },
}

impl MayanStateObject {
    fn latest_package_id(self) -> String {
        match self {
            Self::Direct(fields) => fields.latest_package_id,
            Self::Fields { fields } => fields.latest_package_id,
            Self::Content { content } => content.fields.latest_package_id,
            Self::Data { data } => data.fields.latest_package_id,
        }
    }
}

#[derive(Debug, Deserialize)]
struct MayanStateContent {
    fields: MayanStateFields,
}

#[derive(Debug, Deserialize)]
struct MayanStateFields {
    #[serde(alias = "latestPackageId")]
    latest_package_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mayan_state_object_latest_package_id() {
        let state: MayanStateObject = serde_json::from_value(serde_json::json!({
            "latest_package_id": "0x123"
        }))
        .unwrap();
        assert_eq!(state.latest_package_id(), "0x123");

        let state: MayanStateObject = serde_json::from_value(serde_json::json!({
            "fields": {
                "latest_package_id": "0xabc"
            }
        }))
        .unwrap();
        assert_eq!(state.latest_package_id(), "0xabc");

        let state: MayanStateObject = serde_json::from_value(serde_json::json!({
            "content": {
                "fields": {
                    "latestPackageId": "0xdef"
                }
            }
        }))
        .unwrap();
        assert_eq!(state.latest_package_id(), "0xdef");
    }
}
