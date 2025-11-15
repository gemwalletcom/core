use ::signer::Signer;
use alloy_primitives::hex;
use number_formatter::BigNumberFormatter;
use primitives::{
    ChainSigner, HyperliquidOrder, NumberIncrementer, PerpetualConfirmData, PerpetualDirection, PerpetualModifyConfirmData, PerpetualModifyPositionType,
    PerpetualType, SignerError, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata, stake_type::StakeType, swap::SwapData,
};
use serde::Serialize;
use serde_json::{self, Value};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    core::{
        actions::{
            ApproveAgent, ApproveBuilderFee, Builder, CDeposit, CWithdraw, Cancel, CancelOrder, PlaceOrder, SetReferrer, SpotSend, TokenDelegate,
            UpdateLeverage, WithdrawalRequest, make_market_order, make_position_tp_sl,
        },
        hypercore::{
            approve_agent_typed_data, approve_builder_fee_typed_data, c_deposit_typed_data, c_withdraw_typed_data, cancel_order_typed_data,
            place_order_typed_data, send_spot_token_to_address_typed_data, set_referrer_typed_data, token_delegate_typed_data, update_leverage_typed_data,
            withdrawal_request_typed_data,
        },
    },
    is_spot_swap,
    models::timestamp::TimestampField,
};

const AGENT_NAME_PREFIX: &str = "gemwallet_";
const REFERRAL_CODE: &str = "GEMWALLET";
const BUILDER_ADDRESS: &str = "0x0d9dab1a248f63b0a48965ba8435e4de7497a3dc";
const NATIVE_SPOT_TOKEN: &str = "HYPE:0x0d01dc56dcaaca66ad901c959b4011ec";

type SignerResult<T> = Result<T, SignerError>;

#[derive(Default)]
pub struct HyperCoreSigner;

impl HyperCoreSigner {
    fn sign_transfer_action(&self, input: &TransactionLoadInput, private_key: &[u8]) -> SignerResult<String> {
        let asset = input.input_type.get_asset();
        let amount = BigNumberFormatter::value(&input.value, asset.decimals).map_err(|err| SignerError::InvalidInput(err.to_string()))?;
        self.sign_spot_send(&amount, &input.destination_address, NATIVE_SPOT_TOKEN, private_key)
    }

    fn sign_token_transfer_action(&self, input: &TransactionLoadInput, private_key: &[u8]) -> SignerResult<String> {
        let asset = input.input_type.get_asset();
        let amount = BigNumberFormatter::value(&input.value, asset.decimals).map_err(|err| SignerError::InvalidInput(err.to_string()))?;
        let token_id = asset.id.get_token_id()?;
        self.sign_spot_send(&amount, &input.destination_address, token_id, private_key)
    }

    fn sign_swap_action(&self, input: &TransactionLoadInput, private_key: &[u8]) -> SignerResult<Vec<String>> {
        let swap_data = extract_swap_data(&input.input_type)?;

        if let TransactionInputType::Swap(from_asset, to_asset, _) = &input.input_type
            && is_spot_swap(from_asset.chain(), to_asset.chain())
        {
            let order: PlaceOrder = serde_json::from_str(&swap_data.data.data)?;
            let nonce = Self::timestamp_ms();
            let signature = self.sign_place_order(order, nonce, private_key)?;
            return Ok(vec![signature]);
        }

        let signature = self.sign_typed_action(&swap_data.data.data, private_key)?;
        Ok(vec![signature])
    }

    fn sign_stake_action(&self, input: &TransactionLoadInput, private_key: &[u8]) -> SignerResult<Vec<String>> {
        let stake_type = extract_stake_type(&input.input_type)?;
        let mut nonce_incrementer = NumberIncrementer::new(Self::timestamp_ms());

        match stake_type {
            StakeType::Stake(validator) => {
                let wei = Self::hypercore_wei_from_value(&input.value)?;

                let deposit_request = CDeposit::new(wei, nonce_incrementer.next_val());
                let deposit_action = self.sign_c_deposit(deposit_request, private_key)?;

                let delegate_request = TokenDelegate::new(validator.id.clone(), wei, false, nonce_incrementer.next_val());
                let delegate_action = self.sign_token_delegate(delegate_request, private_key)?;
                Ok(vec![deposit_action, delegate_action])
            }
            StakeType::Unstake(delegation) => {
                let balance = delegation.base.balance.to_string();
                let wei = Self::hypercore_wei_from_value(&balance)?;

                let undelegate_request = TokenDelegate::new(delegation.validator.id.clone(), wei, true, nonce_incrementer.next_val());
                let undelegate_action = self.sign_token_delegate(undelegate_request, private_key)?;

                let withdraw_request = CWithdraw::new(wei, nonce_incrementer.next_val());
                let withdraw_action = self.sign_c_withdraw(withdraw_request, private_key)?;
                Ok(vec![undelegate_action, withdraw_action])
            }
            _ => Err(SignerError::UnsupportedOperation("Stake type not supported".to_string())),
        }
    }

    fn hypercore_wei_from_value(value: &str) -> SignerResult<u64> {
        value
            .parse::<u64>()
            .map_err(|err| SignerError::InvalidInput(format!("Invalid Hypercore wei amount: {err}")))
    }

    fn sign_perpetual_action(&self, input: &TransactionLoadInput, private_key: &[u8]) -> SignerResult<Vec<String>> {
        let perpetual_type = extract_perpetual_type(&input.input_type)?;
        let order = extract_hyperliquid_order(&input.metadata)?;

        let agent_key = hex::decode(&order.agent_private_key).map_err(|_| SignerError::InvalidInput("Invalid agent private key".to_string()))?;
        let builder = get_builder(BUILDER_ADDRESS, order.builder_fee_bps as i32).ok();
        let mut timestamp_incrementer = NumberIncrementer::new(Self::timestamp_ms());
        let mut transactions = Vec::new();

        if order.approve_referral_required {
            transactions.push(self.sign_set_referrer(private_key, REFERRAL_CODE, timestamp_incrementer.next_val())?);
        }

        if order.approve_agent_required {
            transactions.push(self.sign_approve_agent(&order.agent_address, private_key, timestamp_incrementer.next_val())?);
        }

        if order.approve_builder_required {
            transactions.push(self.sign_approve_builder_address(private_key, BUILDER_ADDRESS, order.builder_fee_bps, timestamp_incrementer.next_val())?);
        }

        transactions.extend(self.sign_market_message(perpetual_type, agent_key.as_slice(), builder.as_ref(), &mut timestamp_incrementer)?);

        Ok(transactions)
    }

    fn sign_typed_action(&self, typed_data_json: &str, private_key: &[u8]) -> SignerResult<String> {
        let typed_data: Value = serde_json::from_str(typed_data_json).map_err(|err| SignerError::InvalidInput(format!("Invalid typed data JSON: {err}")))?;

        let message = typed_data
            .get("message")
            .ok_or_else(|| SignerError::InvalidInput("Typed data missing message field".to_string()))?;

        let timestamp = serde_json::from_value::<TimestampField>(message.clone())
            .map_err(|err| SignerError::InvalidInput(format!("Failed to parse time or nonce: {err}")))?;
        let action = serde_json::to_string(message).map_err(|err| SignerError::InvalidInput(format!("Failed to serialize action payload: {err}")))?;

        self.sign_action(typed_data_json, &action, timestamp.value, private_key)
    }

    fn sign_approve_agent(&self, agent_address: &str, private_key: &[u8], timestamp: u64) -> SignerResult<String> {
        let agent_name = format!("{}{}", AGENT_NAME_PREFIX, &agent_address[agent_address.len().saturating_sub(6)..]);
        let agent = ApproveAgent::new(agent_address.to_string(), agent_name, timestamp);
        self.sign_serialized_action(agent, timestamp, private_key, approve_agent_typed_data, "approve agent")
    }

    fn sign_approve_builder_address(&self, agent_key: &[u8], builder_address: &str, rate_bps: u32, timestamp: u64) -> SignerResult<String> {
        let max_fee_rate = fee_rate(rate_bps);
        let request = ApproveBuilderFee::new(max_fee_rate, builder_address.to_string(), timestamp);
        self.sign_serialized_action(request, timestamp, agent_key, approve_builder_fee_typed_data, "approve builder fee")
    }

    fn sign_set_referrer(&self, agent_key: &[u8], code: &str, timestamp: u64) -> SignerResult<String> {
        let referer = SetReferrer::new(code.to_string());
        self.sign_serialized_action(referer, timestamp, agent_key, |value| set_referrer_typed_data(value, timestamp), "set referrer")
    }

    fn sign_spot_send(&self, amount: &str, destination: &str, token: &str, private_key: &[u8]) -> SignerResult<String> {
        let timestamp = Self::timestamp_ms();
        let spot_send = SpotSend::new(amount.to_string(), destination.to_string(), timestamp, token.to_string());
        self.sign_serialized_action(spot_send, timestamp, private_key, send_spot_token_to_address_typed_data, "spot send")
    }

    fn sign_c_deposit(&self, deposit: CDeposit, private_key: &[u8]) -> SignerResult<String> {
        let timestamp = deposit.nonce;
        self.sign_serialized_action(deposit, timestamp, private_key, c_deposit_typed_data, "c deposit")
    }

    fn sign_c_withdraw(&self, withdraw: CWithdraw, private_key: &[u8]) -> SignerResult<String> {
        let timestamp = withdraw.nonce;
        self.sign_serialized_action(withdraw, timestamp, private_key, c_withdraw_typed_data, "c withdraw")
    }

    fn sign_token_delegate(&self, delegate: TokenDelegate, private_key: &[u8]) -> SignerResult<String> {
        let timestamp = delegate.nonce;
        self.sign_serialized_action(delegate, timestamp, private_key, token_delegate_typed_data, "token delegate")
    }

    fn sign_update_leverage(&self, update_leverage: UpdateLeverage, nonce: u64, private_key: &[u8]) -> SignerResult<String> {
        self.sign_serialized_action(
            update_leverage,
            nonce,
            private_key,
            |value| update_leverage_typed_data(value, nonce),
            "update leverage",
        )
    }

    fn sign_market_message(
        &self,
        perpetual_type: &PerpetualType,
        agent_key: &[u8],
        builder: Option<&Builder>,
        timestamp_incrementer: &mut NumberIncrementer,
    ) -> SignerResult<Vec<String>> {
        let (data, is_open) = match perpetual_type {
            PerpetualType::Modify(modify_data) => return self.sign_modify_orders(modify_data, agent_key, builder, timestamp_incrementer),
            PerpetualType::Open(data) => return self.sign_open_orders(data, agent_key, builder, timestamp_incrementer),
            PerpetualType::Increase(data) => (data, true),
            PerpetualType::Close(data) => (data, false),
            PerpetualType::Reduce(reduce_data) => (&reduce_data.data, false),
        };

        let order = Self::market_order_from_confirm_data(data, is_open, builder);
        Ok(vec![self.sign_place_order(order, timestamp_incrementer.next_val(), agent_key)?])
    }

    fn sign_open_orders(
        &self,
        data: &PerpetualConfirmData,
        agent_key: &[u8],
        builder: Option<&Builder>,
        timestamp_incrementer: &mut NumberIncrementer,
    ) -> SignerResult<Vec<String>> {
        let leverage_action = self.sign_update_leverage(
            UpdateLeverage::new(data.asset_index as u32, true, data.leverage),
            timestamp_incrementer.next_val(),
            agent_key,
        )?;
        let place_order_action = self.sign_place_order(
            Self::market_order_from_confirm_data(data, true, builder),
            timestamp_incrementer.next_val(),
            agent_key,
        )?;
        Ok(vec![leverage_action, place_order_action])
    }

    fn sign_modify_orders(
        &self,
        modify_data: &PerpetualModifyConfirmData,
        agent_key: &[u8],
        builder: Option<&Builder>,
        timestamp_incrementer: &mut NumberIncrementer,
    ) -> SignerResult<Vec<String>> {
        modify_data
            .modify_types
            .iter()
            .map(|modify_type| match modify_type {
                PerpetualModifyPositionType::Cancel(orders) => {
                    let cancels = orders.iter().map(|o| CancelOrder::new(o.asset_index as u32, o.order_id)).collect();
                    self.sign_cancel_order(Cancel::new(cancels), timestamp_incrementer.next_val(), agent_key)
                }
                PerpetualModifyPositionType::Tpsl(tpsl) => {
                    let order = make_position_tp_sl(
                        modify_data.asset_index as u32,
                        tpsl.direction == PerpetualDirection::Long,
                        &tpsl.size,
                        tpsl.take_profit.clone(),
                        tpsl.stop_loss.clone(),
                        builder.cloned(),
                    );
                    self.sign_place_order(order, timestamp_incrementer.next_val(), agent_key)
                }
            })
            .collect()
    }

    fn market_order_from_confirm_data(data: &PerpetualConfirmData, is_open: bool, builder: Option<&Builder>) -> PlaceOrder {
        let is_buy = if is_open {
            data.direction == PerpetualDirection::Long
        } else {
            data.direction == PerpetualDirection::Short
        };

        make_market_order(data.asset_index as u32, is_buy, &data.price, &data.size, !is_open, builder.cloned())
    }

    fn sign_place_order(&self, order: PlaceOrder, nonce: u64, private_key: &[u8]) -> SignerResult<String> {
        self.sign_serialized_action(order, nonce, private_key, |value| place_order_typed_data(value, nonce), "place order")
    }

    fn sign_cancel_order(&self, cancel: Cancel, nonce: u64, private_key: &[u8]) -> SignerResult<String> {
        self.sign_serialized_action(cancel, nonce, private_key, |value| cancel_order_typed_data(value, nonce), "cancel order")
    }

    fn sign_action(&self, typed_data: &str, action: &str, timestamp: u64, private_key: &[u8]) -> SignerResult<String> {
        let signature = Signer::sign_eip712(typed_data, private_key).map_err(|err| SignerError::InvalidInput(format!("Failed to sign typed data: {err}")))?;
        self.build_signed_request(signature, action, timestamp)
    }

    fn sign_serialized_action<T, F>(&self, value: T, timestamp: u64, private_key: &[u8], typed_data_fn: F, action_name: &'static str) -> SignerResult<String>
    where
        T: Serialize,
        F: FnOnce(T) -> String,
    {
        let action = serde_json::to_string(&value).map_err(|err| SignerError::InvalidInput(format!("Failed to serialize {action_name} action: {err}")))?;
        let typed_data = typed_data_fn(value);
        self.sign_action(&typed_data, &action, timestamp, private_key)
    }

    fn build_signed_request(&self, signature: String, action: &str, timestamp: u64) -> SignerResult<String> {
        let sig_bytes = hex::decode(&signature).map_err(|err| SignerError::InvalidInput(format!("Invalid signature hex: {err}")))?;

        if sig_bytes.len() < 65 {
            return Err(SignerError::InvalidInput("Signature must be 65 bytes".to_string()));
        }

        let r = hex::encode_prefixed(&sig_bytes[0..32]);
        let s = hex::encode_prefixed(&sig_bytes[32..64]);
        let v = sig_bytes[64] as u64;

        let action_json: Value = serde_json::from_str(action).map_err(|err| SignerError::InvalidInput(format!("Invalid action JSON: {err}")))?;

        let signed_request = serde_json::json!({
            "action": action_json,
            "signature": {
                "r": r,
                "s": s,
                "v": v
            },
            "nonce": timestamp,
            "isFrontend": true
        });

        serde_json::to_string(&signed_request).map_err(|err| SignerError::InvalidInput(format!("Failed to serialize signed request: {err}")))
    }

    fn timestamp_ms() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
    }
}

impl ChainSigner for HyperCoreSigner {
    fn sign_transfer(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<String, SignerError> {
        self.sign_transfer_action(input, private_key)
    }

    fn sign_token_transfer(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<String, SignerError> {
        self.sign_token_transfer_action(input, private_key)
    }

    fn sign_nft_transfer(&self, _input: &TransactionLoadInput, _private_key: &[u8]) -> Result<String, SignerError> {
        Err(SignerError::UnsupportedOperation("NFT transfer not supported".to_string()))
    }

    fn sign_swap(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        self.sign_swap_action(input, private_key)
    }

    fn sign_token_approval(&self, _input: &TransactionLoadInput, _private_key: &[u8]) -> Result<String, SignerError> {
        Err(SignerError::UnsupportedOperation("Token approval not supported".to_string()))
    }

    fn sign_stake(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        self.sign_stake_action(input, private_key)
    }

    fn sign_account_action(&self, _input: &TransactionLoadInput, _private_key: &[u8]) -> Result<String, SignerError> {
        Err(SignerError::UnsupportedOperation("Account action not supported".to_string()))
    }

    fn sign_perpetual(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        self.sign_perpetual_action(input, private_key)
    }

    fn sign_withdrawal(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<String, SignerError> {
        let asset = input.input_type.get_asset();
        let amount = BigNumberFormatter::value(&input.value, asset.decimals).map_err(|err| SignerError::InvalidInput(err.to_string()))?;
        let timestamp = Self::timestamp_ms();

        let withdrawal_request = WithdrawalRequest::new(amount, timestamp, input.destination_address.clone());
        self.sign_serialized_action(withdrawal_request, timestamp, private_key, withdrawal_request_typed_data, "withdrawal")
    }

    fn sign_data(&self, _input: &TransactionLoadInput, _private_key: &[u8]) -> Result<String, SignerError> {
        Err(SignerError::UnsupportedOperation("Data signing not supported".to_string()))
    }
}

fn extract_swap_data(input_type: &TransactionInputType) -> Result<&SwapData, SignerError> {
    if let TransactionInputType::Swap(_, _, swap_data) = input_type {
        Ok(swap_data)
    } else {
        Err(SignerError::InvalidInput("Expected Swap input".to_string()))
    }
}

fn extract_stake_type(input_type: &TransactionInputType) -> Result<&StakeType, SignerError> {
    if let TransactionInputType::Stake(_, stake_type) = input_type {
        Ok(stake_type)
    } else {
        Err(SignerError::InvalidInput("Expected Stake input".to_string()))
    }
}

fn extract_perpetual_type(input_type: &TransactionInputType) -> Result<&PerpetualType, SignerError> {
    if let TransactionInputType::Perpetual(_, perpetual_type) = input_type {
        Ok(perpetual_type)
    } else {
        Err(SignerError::InvalidInput("Expected Perpetual input".to_string()))
    }
}

fn extract_hyperliquid_order(metadata: &TransactionLoadMetadata) -> Result<&HyperliquidOrder, SignerError> {
    if let TransactionLoadMetadata::Hyperliquid { order: Some(order) } = metadata {
        Ok(order)
    } else {
        Err(SignerError::InvalidInput("Hyperliquid order metadata required".to_string()))
    }
}

fn get_builder(builder: &str, fee: i32) -> Result<Builder, SignerError> {
    if fee < 0 {
        return Err(SignerError::InvalidInput("Builder fee cannot be negative".to_string()));
    }
    Ok(Builder {
        builder_address: builder.to_string(),
        fee: fee as u32,
    })
}

fn fee_rate(tenths_bps: u32) -> String {
    format!("{}%", (tenths_bps as f64) * 0.001)
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::{BigInt, BigUint};
    use primitives::{
        Asset, Chain, Delegation, DelegationBase, DelegationState, DelegationValidator, GasPriceType, StakeType, TransactionInputType, TransactionLoadInput,
        TransactionLoadMetadata,
    };

    #[test]
    fn unstake_actions_have_unique_nonces() {
        let signer = HyperCoreSigner::default();
        let asset = Asset::from_chain(Chain::HyperCore);
        let delegation = Delegation {
            base: DelegationBase {
                asset_id: asset.id.clone(),
                state: DelegationState::Active,
                balance: BigUint::from(150_000_000u64),
                shares: BigUint::from(0u64),
                rewards: BigUint::from(0u64),
                completion_date: None,
                delegation_id: "delegation".into(),
                validator_id: "validator".into(),
            },
            validator: DelegationValidator {
                chain: Chain::HyperCore,
                id: "0x66be52ec79f829cc88e5778a255e2cb9492798fd".into(),
                name: "Validator".into(),
                is_active: true,
                commission: 0.0,
                apr: 0.0,
            },
            price: None,
        };
        let input = TransactionLoadInput {
            input_type: TransactionInputType::Stake(asset, StakeType::Unstake(delegation)),
            sender_address: "0xsender".into(),
            destination_address: "".into(),
            value: "0".into(),
            gas_price: GasPriceType::regular(BigInt::from(0)),
            memo: None,
            is_max_value: false,
            metadata: TransactionLoadMetadata::None,
        };
        let private_key = [1u8; 32];

        let responses = signer.sign_stake_action(&input, &private_key).expect("should sign");
        assert_eq!(responses.len(), 2);

        let nonces: Vec<u64> = responses
            .iter()
            .map(|payload| {
                let value: serde_json::Value = serde_json::from_str(payload).expect("valid json");
                value["action"]["nonce"].as_u64().expect("action nonce")
            })
            .collect();

        assert_eq!(nonces.len(), 2);
        assert!(nonces[0] < nonces[1], "unstake actions should advance nonce");
    }

    #[test]
    fn hypercore_wei_parser_parses_amount() {
        let wei = HyperCoreSigner::hypercore_wei_from_value("150000000").unwrap();
        assert_eq!(wei, 150000000);
    }

    #[test]
    fn hypercore_wei_parser_rejects_invalid_inputs() {
        assert!(HyperCoreSigner::hypercore_wei_from_value("invalid").is_err());
        assert!(HyperCoreSigner::hypercore_wei_from_value("-1").is_err());
        assert!(HyperCoreSigner::hypercore_wei_from_value("1.23").is_err());
        let too_large = (u64::MAX as u128 + 1).to_string();
        assert!(HyperCoreSigner::hypercore_wei_from_value(&too_large).is_err());
    }

    #[test]
    fn market_order_from_open_long_sets_buy() {
        let data = PerpetualConfirmData::mock(PerpetualDirection::Long, 11);
        let builder = Builder {
            builder_address: "0xdeadbeef".to_string(),
            fee: 25,
        };

        let order = HyperCoreSigner::market_order_from_confirm_data(&data, true, Some(&builder));

        assert_eq!(order.orders.len(), 1);
        let market_order = &order.orders[0];
        assert!(market_order.is_buy);
        assert!(!market_order.reduce_only);
        assert_eq!(market_order.asset, data.asset_index as u32);
        assert_eq!(market_order.size, data.size);

        let cloned_builder = order.builder.expect("builder should be propagated");
        assert_eq!(cloned_builder.builder_address, builder.builder_address);
        assert_eq!(cloned_builder.fee, builder.fee);
    }

    #[test]
    fn market_order_from_close_short_sets_sell_and_reduce_only() {
        let data = PerpetualConfirmData::mock(PerpetualDirection::Short, 5);
        let order = HyperCoreSigner::market_order_from_confirm_data(&data, false, None);

        let market_order = &order.orders[0];
        assert!(market_order.is_buy);
        assert!(market_order.reduce_only);
    }

    #[test]
    fn market_order_from_open_short_sets_sell() {
        let data = PerpetualConfirmData::mock(PerpetualDirection::Short, 9);
        let order = HyperCoreSigner::market_order_from_confirm_data(&data, true, None);

        let market_order = &order.orders[0];
        assert!(!market_order.is_buy);
        assert!(!market_order.reduce_only);
    }
}
