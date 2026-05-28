use super::sui_error;
use crate::{
    SwapperError,
    mayan::{
        constants::{SUI_CCTP_CORE_STATE, SUI_CCTP_DENY_LIST, SUI_CCTP_TOKEN_STATE, SUI_MCTP_FEE_MANAGER_STATE, SUI_MCTP_STATE, SUI_WORMHOLE_STATE},
        model::MayanMctpQuote,
    },
};
use futures::try_join;
use gem_sui::{
    SuiClient, is_sui_coin,
    models::{Coin, OwnedCoins},
    tx_builder::{ResolvedObjectInput, TransactionBuilderInput},
};
use serde::Deserialize;
use std::collections::HashMap;

pub(super) struct PrefetchedSuiData {
    pub(super) transaction: TransactionBuilderInput,
    pub(super) input_coins: OwnedCoins<Coin>,
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
        let has_auction = route.has_auction == Some(true);
        let transaction = async { TransactionBuilderInput::prefetch(client, sender, gas_budget).await.map_err(sui_error) };
        let input_coins = async {
            if route.from_token.contract.as_str() != mctp_input_contract.as_str() || is_sui_coin(&mctp_input_contract) {
                Ok(OwnedCoins::default())
            } else {
                client.get_coins(sender, &mctp_input_contract).await.map_err(sui_error)
            }
        };
        let object_ids = sui_object_ids(has_auction, &from_token_verified_address, &mctp_verified_input_address, &mctp_input_treasury);
        let resolved_objects = async {
            let objects = ResolvedObjectInput::get_multiple(client, object_ids.clone()).await.map_err(sui_error)?;
            if objects.len() != object_ids.len() {
                return Err(SwapperError::transaction_error("Failed to prefetch all Mayan Sui objects"));
            }
            Ok(objects)
        };
        let mctp_package_id = get_mayan_sui_package_id(client, SUI_MCTP_STATE);
        let fee_manager_package_id = async {
            if has_auction {
                get_mayan_sui_package_id(client, SUI_MCTP_FEE_MANAGER_STATE).await.map(Some)
            } else {
                Ok(None)
            }
        };
        let (transaction, input_coins, resolved_objects, mctp_package_id, fee_manager_package_id) =
            try_join!(transaction, input_coins, resolved_objects, mctp_package_id, fee_manager_package_id)?;
        let objects = object_ids.into_iter().zip(resolved_objects).collect();

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

fn sui_object_ids(has_auction: bool, from_token_verified_address: &str, mctp_verified_input_address: &str, mctp_input_treasury: &str) -> Vec<String> {
    let mut object_ids = vec![
        SUI_MCTP_STATE.to_string(),
        SUI_CCTP_CORE_STATE.to_string(),
        SUI_CCTP_TOKEN_STATE.to_string(),
        SUI_CCTP_DENY_LIST.to_string(),
        SUI_WORMHOLE_STATE.to_string(),
        from_token_verified_address.to_string(),
        mctp_verified_input_address.to_string(),
        mctp_input_treasury.to_string(),
    ];
    if has_auction {
        object_ids.push(SUI_MCTP_FEE_MANAGER_STATE.to_string());
    }
    object_ids
}

async fn get_mayan_sui_package_id(client: &SuiClient, state_object_id: &str) -> Result<String, SwapperError> {
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
