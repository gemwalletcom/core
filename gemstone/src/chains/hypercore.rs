use crate::{
    GemstoneError,
    alien::AlienSigner,
    hyperliquid::{HyperCore, HyperCoreModelFactory},
    models::transaction::{GemPerpetualType, GemStakeType, GemTransactionLoadInput, GemTransactionLoadMetadata},
};
use gem_hypercore::core::actions::Builder;
use number_formatter::BigNumberFormatter;
use primitives::{NumberIncrementer, PerpetualDirection, SignerError};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[uniffi::export(with_foreign)]
pub trait ChainSigner: Send + Sync {
    fn sign_transfer(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<String, GemstoneError>;
    fn sign_token_transfer(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<String, GemstoneError>;
    fn sign_nft_transfer(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<String, GemstoneError>;
    fn sign_swap(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<Vec<String>, GemstoneError>;
    fn sign_token_approval(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<String, GemstoneError>;
    fn sign_stake(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<Vec<String>, GemstoneError>;
    fn sign_account_action(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<String, GemstoneError>;
    fn sign_perpetual(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<Vec<String>, GemstoneError>;
    fn sign_withdrawal(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<String, GemstoneError>;
    fn sign_data(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<String, GemstoneError>;
}

const AGENT_NAME_PREFIX: &str = "gemwallet_";
const REFERRAL_CODE: &str = "GEMWALLET";
const BUILDER_ADDRESS: &str = "0x0d9dab1a248f63b0a48965ba8435e4de7497a3dc";
const NATIVE_SPOT_TOKEN: &str = "HYPE:0x0d01dc56dcaaca66ad901c959b4011ec";

#[derive(uniffi::Object)]
pub struct GemSigner {
    hyper_core: Arc<HyperCore>,
    factory: Arc<HyperCoreModelFactory>,
}

#[uniffi::export]
impl GemSigner {
    #[uniffi::constructor]
    pub fn new(signer: Arc<dyn AlienSigner>) -> Self {
        Self {
            hyper_core: Arc::new(HyperCore::new(signer)),
            factory: Arc::new(HyperCoreModelFactory::new()),
        }
    }
}

#[uniffi::export]
impl ChainSigner for GemSigner {
    fn sign_transfer(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        let asset = input.input_type.asset();
        let amount = BigNumberFormatter::value(&input.value, asset.decimals)?;
        self.sign_spot_send(&amount, &input.destination_address, NATIVE_SPOT_TOKEN, private_key)
    }

    fn sign_token_transfer(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        let asset = input.input_type.asset();
        let amount = BigNumberFormatter::value(&input.value, asset.decimals)?;
        let token_id = asset.id.get_token_id()?;
        self.sign_spot_send(&amount, &input.destination_address, token_id, private_key)
    }

    fn sign_swap(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<Vec<String>, GemstoneError> {
        let swap_data = input.input_type.swap_data()?;
        let signature = self.hyper_core.sign_typed_action(swap_data.data.data.clone(), private_key)?;
        Ok(vec![signature])
    }

    fn sign_stake(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<Vec<String>, GemstoneError> {
        let stake_type = input.input_type.stake_type()?;
        let mut nonce_incrementer = NumberIncrementer::new(Self::get_timestamp_ms());

        match stake_type {
            GemStakeType::Delegate { validator } => {
                let wei = BigNumberFormatter::value_as_u64(&input.value, 10)?;

                let deposit_request = self.factory.make_transfer_to_staking(wei, nonce_incrementer.next_val());
                let deposit_action = self.hyper_core.sign_c_deposit(deposit_request, private_key.clone())?;

                let delegate_request = self.factory.make_delegate(validator.id.clone(), wei, nonce_incrementer.next_val());
                let delegate_action = self.hyper_core.sign_token_delegate(delegate_request, private_key)?;
                Ok(vec![deposit_action, delegate_action])
            }
            GemStakeType::Undelegate { delegation } => {
                let wei = BigNumberFormatter::value_as_u64(&delegation.base.balance.to_string(), 10)?;

                let undelegate_request = self.factory.make_undelegate(delegation.validator.id.clone(), wei, nonce_incrementer.current());
                let undelegate_action = self.hyper_core.sign_token_delegate(undelegate_request, private_key.clone())?;

                let withdraw_request = self.factory.make_withdraw_from_staking(wei, nonce_incrementer.next_val());
                let withdraw_action = self.hyper_core.sign_c_withdraw(withdraw_request, private_key)?;
                Ok(vec![undelegate_action, withdraw_action])
            }
            _ => Err(SignerError::UnsupportedOperation("Stake type not supported".to_string()).into()),
        }
    }

    fn sign_nft_transfer(&self, _input: GemTransactionLoadInput, _private_key: Vec<u8>) -> Result<String, GemstoneError> {
        Err(SignerError::UnsupportedOperation("NFT transfer not supported".to_string()).into())
    }

    fn sign_token_approval(&self, _input: GemTransactionLoadInput, _private_key: Vec<u8>) -> Result<String, GemstoneError> {
        Err(SignerError::UnsupportedOperation("Token approval not supported".to_string()).into())
    }

    fn sign_account_action(&self, _input: GemTransactionLoadInput, _private_key: Vec<u8>) -> Result<String, GemstoneError> {
        Err(SignerError::UnsupportedOperation("Account action not supported".to_string()).into())
    }

    fn sign_perpetual(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<Vec<String>, GemstoneError> {
        let perpetual_type = input.input_type.perpetual_type()?;
        let order = match &input.metadata {
            GemTransactionLoadMetadata::Hyperliquid { order: Some(order) } => order,
            _ => return Err(SignerError::InvalidInput("Hyperliquid order metadata required".to_string()).into()),
        };

        let agent_key = hex::decode(&order.agent_private_key).map_err(|_| SignerError::InvalidInput("Invalid agent private key".to_string()))?;
        let builder = Self::get_builder(BUILDER_ADDRESS, order.builder_fee_bps as i32).ok();
        let mut timestamp_incrementer = NumberIncrementer::new(Self::get_timestamp_ms());
        let mut transactions = Vec::new();

        if order.approve_referral_required {
            transactions.push(self.sign_set_referer(&private_key, REFERRAL_CODE, timestamp_incrementer.next_val())?);
        }

        if order.approve_agent_required {
            transactions.push(self.sign_approve_agent(&order.agent_address, &private_key, timestamp_incrementer.next_val())?);
        }

        if order.approve_builder_required {
            transactions.push(self.sign_approve_builder_address(&private_key, BUILDER_ADDRESS, order.builder_fee_bps, timestamp_incrementer.next_val())?);
        }

        transactions.push(self.sign_market_message(&perpetual_type, &agent_key, builder, timestamp_incrementer.next_val())?);

        Ok(transactions)
    }

    fn sign_withdrawal(&self, _input: GemTransactionLoadInput, _private_key: Vec<u8>) -> Result<String, GemstoneError> {
        Err(SignerError::UnsupportedOperation("Withdrawal not supported".to_string()).into())
    }

    fn sign_data(&self, _input: GemTransactionLoadInput, _private_key: Vec<u8>) -> Result<String, GemstoneError> {
        Err(SignerError::UnsupportedOperation("Data signing not supported".to_string()).into())
    }
}

impl GemSigner {
    pub fn sign_approve_agent(&self, agent_address: &str, private_key: &[u8], timestamp: u64) -> Result<String, GemstoneError> {
        let agent_name = format!("{}{}", AGENT_NAME_PREFIX, &agent_address[agent_address.len() - 6..]);
        let agent = self.factory.make_approve_agent(agent_name, agent_address.to_string(), timestamp);
        self.hyper_core.sign_approve_agent(agent, private_key.to_vec())
    }

    fn sign_approve_builder_address(&self, agent_key: &[u8], builder_address: &str, rate_bps: u32, timestamp: u64) -> Result<String, GemstoneError> {
        let max_fee_rate = Self::fee_rate(rate_bps);
        let request = self.factory.make_approve_builder(max_fee_rate, builder_address.to_string(), timestamp);
        self.hyper_core.sign_approve_builder_fee(request, agent_key.to_vec())
    }

    fn sign_set_referer(&self, agent_key: &[u8], code: &str, timestamp: u64) -> Result<String, GemstoneError> {
        let referer = self.factory.make_set_referrer(code.to_string());
        self.hyper_core.sign_set_referrer(referer, timestamp, agent_key.to_vec())
    }

    fn sign_spot_send(&self, amount: &str, destination: &str, token: &str, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        let timestamp = Self::get_timestamp_ms();
        let spot_send = self
            .factory
            .send_spot_token_to_address(amount.to_string(), destination.to_lowercase(), timestamp, token.to_string());
        self.hyper_core.sign_spot_send(spot_send, private_key)
    }

    fn sign_market_message(
        &self,
        perpetual_type: &GemPerpetualType,
        agent_key: &[u8],
        builder: Option<Builder>,
        timestamp: u64,
    ) -> Result<String, GemstoneError> {
        let (data, is_open) = match perpetual_type {
            GemPerpetualType::Open(data) => (data, true),
            GemPerpetualType::Close(data) => (data, false),
        };

        let is_buy = if is_open {
            match data.direction {
                PerpetualDirection::Long => true,
                PerpetualDirection::Short => false,
            }
        } else {
            match data.direction {
                PerpetualDirection::Long => false,
                PerpetualDirection::Short => true,
            }
        };

        let order = self
            .factory
            .make_market_order(data.asset_index as u32, is_buy, data.price.clone(), data.size.clone(), !is_open, builder);

        self.hyper_core.sign_place_order(order, timestamp, agent_key.to_vec())
    }

    fn fee_rate(tenths_bps: u32) -> String {
        format!("{}%", (tenths_bps as f64) * 0.001)
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

    fn get_timestamp_ms() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
    }
}
