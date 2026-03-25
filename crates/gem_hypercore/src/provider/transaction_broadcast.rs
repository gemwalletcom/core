use alloy_primitives::{Address, hex};
use async_trait::async_trait;
use chain_traits::{ChainTransactionBroadcast, ChainTransactionDecode};
use gem_hash::keccak::keccak256;
use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
use serde::Deserialize;
use std::error::Error;

use gem_client::Client;
use primitives::BroadcastOptions;

use crate::{
    core::{
        actions::{
            ApproveAgent, ApproveBuilderFee, CDeposit, CWithdraw, Cancel, PlaceOrder, SetReferrer, SpotSend, TokenDelegate, UpdateLeverage, UsdClassTransfer, UsdSend,
            WithdrawalRequest,
        },
        hypercore::{
            approve_agent_typed_data, approve_builder_fee_typed_data, c_deposit_typed_data, c_withdraw_typed_data, cancel_order_typed_data, place_order_typed_data,
            send_perps_usd_to_address_typed_data, send_spot_token_to_address_typed_data, set_referrer_typed_data, token_delegate_typed_data, transfer_perps_to_spot_typed_data,
            update_leverage_typed_data, withdrawal_request_typed_data,
        },
    },
    models::action::{ACTION_ID_PREFIX, ExchangeRequest},
    provider::{
        BroadcastProvider,
        transactions_mapper::{map_transaction_broadcast, map_transaction_broadcast_from_str},
    },
    rpc::client::HyperCoreClient,
};

#[async_trait]
impl<C: Client> ChainTransactionBroadcast for HyperCoreClient<C> {
    async fn transaction_broadcast(&self, data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        let request = serde_json::from_str(&data)?;
        let response = self.exchange(request).await?;
        let transaction_id = map_transaction_broadcast(response, &data)?;
        let _ = cache_transaction_sender(self, &data, &transaction_id);
        Ok(transaction_id)
    }
}

impl ChainTransactionDecode for BroadcastProvider {
    fn decode_transaction_broadcast(&self, response: &str) -> Option<String> {
        map_transaction_broadcast_from_str(response).ok()
    }
}

#[derive(Debug, Deserialize)]
struct SignedExchangeRequest {
    action: serde_json::Value,
    nonce: u64,
    signature: SignedExchangeSignature,
}

#[derive(Debug, Deserialize)]
struct SignedExchangeSignature {
    r: String,
    s: String,
    v: u8,
}

fn cache_transaction_sender<C: Client>(client: &HyperCoreClient<C>, data: &str, transaction_id: &str) -> Result<(), Box<dyn Error + Sync + Send>> {
    let signer = recover_sender_address(data)?;
    let sender = client.get_cached_agent_owner(&signer)?.unwrap_or(signer);
    client.cache_transaction_sender(transaction_id, &sender)?;

    if let Some(nonce) = ExchangeRequest::get_nonce(data) {
        client.cache_transaction_sender(&format!("{ACTION_ID_PREFIX}{nonce}"), &sender)?;
    }

    Ok(())
}

fn recover_sender_address(data: &str) -> Result<String, Box<dyn Error + Sync + Send>> {
    let request: SignedExchangeRequest = serde_json::from_str(data)?;
    let typed_data = typed_data_for_request(&request)?;
    let digest = signer::hash_eip712(&typed_data)?;
    let signature = signature_from_request(&request.signature)?;
    let recovery_id = recovery_id_from_v(request.signature.v)?;
    let public_key = VerifyingKey::recover_from_prehash(&digest, &signature, recovery_id)?;
    let encoded_point = public_key.to_encoded_point(false);
    let hash = keccak256(&encoded_point.as_bytes()[1..]);
    Ok(Address::from_slice(&hash[12..]).to_string().to_lowercase())
}

fn typed_data_for_request(request: &SignedExchangeRequest) -> Result<String, Box<dyn Error + Sync + Send>> {
    let action_type = request.action.get("type").and_then(serde_json::Value::as_str).ok_or("Missing action type")?;

    Ok(match action_type {
        "order" => place_order_typed_data(serde_json::from_value::<PlaceOrder>(request.action.clone())?, request.nonce),
        "setReferrer" => set_referrer_typed_data(serde_json::from_value::<SetReferrer>(request.action.clone())?, request.nonce),
        "updateLeverage" => update_leverage_typed_data(serde_json::from_value::<UpdateLeverage>(request.action.clone())?, request.nonce),
        "cancel" => cancel_order_typed_data(serde_json::from_value::<Cancel>(request.action.clone())?, request.nonce),
        "withdraw3" => withdrawal_request_typed_data(serde_json::from_value::<WithdrawalRequest>(request.action.clone())?),
        "approveAgent" => approve_agent_typed_data(serde_json::from_value::<ApproveAgent>(request.action.clone())?),
        "approveBuilderFee" => approve_builder_fee_typed_data(serde_json::from_value::<ApproveBuilderFee>(request.action.clone())?),
        "spotSend" => send_spot_token_to_address_typed_data(serde_json::from_value::<SpotSend>(request.action.clone())?),
        "usdSend" => send_perps_usd_to_address_typed_data(serde_json::from_value::<UsdSend>(request.action.clone())?),
        "usdClassTransfer" => transfer_perps_to_spot_typed_data(serde_json::from_value::<UsdClassTransfer>(request.action.clone())?),
        "cDeposit" => c_deposit_typed_data(serde_json::from_value::<CDeposit>(request.action.clone())?),
        "cWithdraw" => c_withdraw_typed_data(serde_json::from_value::<CWithdraw>(request.action.clone())?),
        "tokenDelegate" => token_delegate_typed_data(serde_json::from_value::<TokenDelegate>(request.action.clone())?),
        other => return Err(format!("Unsupported Hypercore action type: {other}").into()),
    })
}

fn signature_from_request(signature: &SignedExchangeSignature) -> Result<Signature, Box<dyn Error + Sync + Send>> {
    let r = hex::decode(signature.r.trim_start_matches("0x"))?;
    let s = hex::decode(signature.s.trim_start_matches("0x"))?;
    let r: [u8; 32] = r.try_into().map_err(|_| "Invalid r length")?;
    let s: [u8; 32] = s.try_into().map_err(|_| "Invalid s length")?;
    Ok(Signature::from_scalars(r, s)?)
}

fn recovery_id_from_v(v: u8) -> Result<RecoveryId, Box<dyn Error + Sync + Send>> {
    let normalized = if v >= 27 { v - 27 } else { v };
    Ok(RecoveryId::try_from(normalized)?)
}

#[cfg(test)]
mod tests {
    use super::{cache_transaction_sender, recover_sender_address};
    use crate::rpc::client::HyperCoreClient;
    use gem_client::testkit::MockClient;

    #[test]
    fn test_recover_sender_address() {
        let request = include_str!("../../testdata/hl_action_open_long_order.json").trim();
        let address = recover_sender_address(request).unwrap();

        assert_eq!(address, "0xbbb0187503c3b5f08b03d674b9ac86ec30d790d2");
    }

    #[test]
    fn test_cache_transaction_sender_uses_cached_agent_owner() {
        let client = HyperCoreClient::<MockClient>::mock();
        client
            .cache_agent_owner("0xbbb0187503c3b5f08b03d674b9ac86ec30d790d2", "0xba4d1d35bce0e8f28e5a3403e7a0b996c5d50ac4")
            .unwrap();

        cache_transaction_sender(&client, include_str!("../../testdata/hl_action_open_long_order.json").trim(), "187530505765").unwrap();

        assert_eq!(
            client.get_cached_transaction_sender("187530505765").unwrap().as_deref(),
            Some("0xba4d1d35bce0e8f28e5a3403e7a0b996c5d50ac4")
        );
    }
}
