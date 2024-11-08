use gem_sui::model::{Coin, Gas, Object, StakeInput, TokenTransferInput, TransferInput, TxOutput, UnstakeInput};

#[derive(uniffi::Record, Clone)]
pub struct SuiCoin {
    pub coin_type: String,
    pub balance: u64,
    pub object_ref: SuiObjectRef,
}

impl From<SuiCoin> for Coin {
    fn from(value: SuiCoin) -> Self {
        Coin {
            coin_type: value.coin_type,
            balance: value.balance,
            object: value.object_ref.into(),
        }
    }
}

impl From<&SuiCoin> for Coin {
    fn from(value: &SuiCoin) -> Self {
        value.to_owned().into()
    }
}

#[derive(uniffi::Record, Clone)]
pub struct SuiObjectRef {
    pub object_id: String,
    pub digest: String,
    pub version: u64,
}

impl From<SuiObjectRef> for Object {
    fn from(value: SuiObjectRef) -> Self {
        Object {
            object_id: value.object_id,
            digest: value.digest,
            version: value.version,
        }
    }
}

#[derive(uniffi::Record, Clone)]
pub struct SuiGas {
    pub budget: u64,
    pub price: u64,
}

impl From<SuiGas> for Gas {
    fn from(value: SuiGas) -> Self {
        Gas {
            budget: value.budget,
            price: value.price,
        }
    }
}

#[derive(uniffi::Record, Clone)]
pub struct SuiStakeInput {
    pub sender: String,
    pub validator: String,
    pub stake_amount: u64,
    pub gas: SuiGas,
    pub coins: Vec<SuiCoin>,
}

impl From<&SuiStakeInput> for StakeInput {
    fn from(value: &SuiStakeInput) -> Self {
        Self {
            sender: value.sender.clone(),
            validator: value.validator.clone(),
            stake_amount: value.stake_amount,
            gas: value.gas.clone().into(),
            coins: value.coins.clone().into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(uniffi::Record, Clone)]
pub struct SuiUnstakeInput {
    pub sender: String,
    pub staked_sui: SuiObjectRef,
    pub gas: SuiGas,
    pub gas_coin: SuiCoin,
}

impl From<&SuiUnstakeInput> for UnstakeInput {
    fn from(value: &SuiUnstakeInput) -> Self {
        Self {
            sender: value.sender.clone(),
            staked_sui: value.staked_sui.clone().into(),
            gas: value.gas.clone().into(),
            gas_coin: value.gas_coin.clone().into(),
        }
    }
}

#[derive(uniffi::Record, Clone)]
pub struct SuiTransferInput {
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
    pub coins: Vec<SuiCoin>,
    pub send_max: bool,
    pub gas: SuiGas,
}

impl From<&SuiTransferInput> for TransferInput {
    fn from(value: &SuiTransferInput) -> Self {
        Self {
            sender: value.sender.clone(),
            recipient: value.recipient.clone(),
            amount: value.amount,
            coins: value.coins.clone().into_iter().map(Into::into).collect(),
            send_max: value.send_max,
            gas: value.gas.clone().into(),
        }
    }
}

#[derive(uniffi::Record, Clone)]
pub struct SuiTokenTransferInput {
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
    pub tokens: Vec<SuiCoin>,
    pub gas: SuiGas,
    pub gas_coin: SuiCoin,
}

impl From<&SuiTokenTransferInput> for TokenTransferInput {
    fn from(value: &SuiTokenTransferInput) -> Self {
        Self {
            sender: value.sender.clone(),
            recipient: value.recipient.clone(),
            amount: value.amount,
            tokens: value.tokens.clone().into_iter().map(Into::into).collect(),
            gas: value.gas.clone().into(),
            gas_coin: value.gas_coin.clone().into(),
        }
    }
}

#[derive(uniffi::Record, Clone)]
pub struct SuiTxOutput {
    pub tx_data: Vec<u8>,
    pub hash: Vec<u8>,
}

impl From<TxOutput> for SuiTxOutput {
    fn from(value: TxOutput) -> Self {
        Self {
            tx_data: value.tx_data,
            hash: value.hash,
        }
    }
}
