use ::signer::Signer;
use alloy_primitives::hex;
use number_formatter::BigNumberFormatter;
use primitives::{
    ChainSigner, HyperliquidOrder, NumberIncrementer, PerpetualDirection, PerpetualType, SignerError, TransactionInputType, TransactionLoadInput,
    TransactionLoadMetadata, stake_type::StakeType, swap::SwapData,
};
use serde::Serialize;
use serde_json::{self, Value};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::{
    actions::{
        ApproveAgent, ApproveBuilderFee, Builder, CDeposit, CWithdraw, PlaceOrder, SetReferrer, SpotSend, TokenDelegate, WithdrawalRequest, make_market_order,
    },
    hypercore::{
        approve_agent_typed_data, approve_builder_fee_typed_data, c_deposit_typed_data, c_withdraw_typed_data, place_order_typed_data,
        send_spot_token_to_address_typed_data, set_referrer_typed_data, token_delegate_typed_data, withdrawal_request_typed_data,
    },
};
use crate::models::timestamp::TimestampField;

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
        let signature = self.sign_typed_action(&swap_data.data.data, private_key)?;
        Ok(vec![signature])
    }

    fn sign_stake_action(&self, input: &TransactionLoadInput, private_key: &[u8]) -> SignerResult<Vec<String>> {
        let stake_type = extract_stake_type(&input.input_type)?;
        let mut nonce_incrementer = NumberIncrementer::new(Self::timestamp_ms());

        match stake_type {
            StakeType::Stake(validator) => {
                let wei = BigNumberFormatter::value_as_u64(&input.value, 10).map_err(|err| SignerError::InvalidInput(err.to_string()))?;

                let deposit_request = CDeposit::new(wei, nonce_incrementer.next_val());
                let deposit_action = self.sign_c_deposit(deposit_request, private_key)?;

                let delegate_request = TokenDelegate::new(validator.id.clone(), wei, false, nonce_incrementer.next_val());
                let delegate_action = self.sign_token_delegate(delegate_request, private_key)?;
                Ok(vec![deposit_action, delegate_action])
            }
            StakeType::Unstake(delegation) => {
                let wei =
                    BigNumberFormatter::value_as_u64(&delegation.base.balance.to_string(), 10).map_err(|err| SignerError::InvalidInput(err.to_string()))?;

                let undelegate_request = TokenDelegate::new(delegation.validator.id.clone(), wei, true, nonce_incrementer.current());
                let undelegate_action = self.sign_token_delegate(undelegate_request, private_key)?;

                let withdraw_request = CWithdraw::new(wei, nonce_incrementer.next_val());
                let withdraw_action = self.sign_c_withdraw(withdraw_request, private_key)?;
                Ok(vec![undelegate_action, withdraw_action])
            }
            _ => Err(SignerError::UnsupportedOperation("Stake type not supported".to_string())),
        }
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

        transactions.push(self.sign_market_message(perpetual_type, agent_key.as_slice(), builder, timestamp_incrementer.next_val())?);

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

    fn sign_market_message(&self, perpetual_type: &PerpetualType, agent_key: &[u8], builder: Option<Builder>, timestamp: u64) -> SignerResult<String> {
        let (data, is_open) = match perpetual_type {
            PerpetualType::Open(data) | PerpetualType::Increase(data) => (data, true),
            PerpetualType::Close(data) => (data, false),
            PerpetualType::Reduce(reduce_data) => (&reduce_data.data, false),
        };

        let is_buy = if is_open {
            matches!(data.direction, PerpetualDirection::Long)
        } else {
            matches!(data.direction, PerpetualDirection::Short)
        };

        let order = make_market_order(data.asset_index as u32, is_buy, &data.price, &data.size, !is_open, builder);
        self.sign_place_order(order, timestamp, agent_key)
    }

    fn sign_place_order(&self, order: PlaceOrder, nonce: u64, private_key: &[u8]) -> SignerResult<String> {
        self.sign_serialized_action(order, nonce, private_key, |value| place_order_typed_data(value, nonce), "place order")
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
