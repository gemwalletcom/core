use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use gem_ton::stonfi::constants::{DEX_OP_CODES, TX_DEADLINE};
use num_bigint::{BigInt, BigUint};
use thiserror::Error;
use tonlib_client::{
    client::TonClient,
    contract::{JettonMasterContract, TonContractError, TonContractFactory},
};
use tonlib_core::{
    cell::{ArcCell, Cell, CellBuilder, TonCellError},
    TonAddress,
};

pub struct ProxyTonTransferParams {
    pub query_id: u64,
    pub ton_amount: u128,
    pub destination_address: TonAddress,
    pub refund_address: TonAddress,
    pub forward_payload: ArcCell,
    pub forward_ton_amount: u128,
}

#[async_trait]
pub trait AbstractPton: Send + Sync + Debug {
    async fn get_ton_transfer_params(&self, body: ProxyTonTransferParams) -> Result<SenderArguments, StonfiBaseRouterError>;
}

#[derive(Debug)]
pub struct SwapBodyParams {
    pub ask_jetton_wallet_address: TonAddress,
    pub receiver_address: TonAddress,
    pub min_ask_amount: u128,
    pub refund_address: TonAddress,
    pub excesses_address: Option<TonAddress>,
    pub dex_custom_payload: Option<ArcCell>,
    pub dex_custom_payload_forward_gas_amount: Option<u128>,
    pub refund_payload: Option<ArcCell>,
    pub refund_forward_gas_amount: Option<u128>,
    pub referral_address: Option<TonAddress>,
    pub referral_value: Option<u16>,
    pub deadline: Option<u64>,
}

pub struct SwapTonToJettonTxParams {
    pub user_wallet_address: TonAddress,
    pub receiver_address: Option<TonAddress>,
    pub proxy_ton: Arc<dyn AbstractPton>,
    pub ask_jetton_address: TonAddress,
    pub offer_ammount: u128,
    pub min_ask_amount: u128,
    pub refund_address: Option<TonAddress>,
    pub excesses_address: Option<TonAddress>,
    pub dex_custom_payload: Option<ArcCell>,
    pub dex_custom_payload_forward_gas_amount: Option<u128>,
    pub refund_payload: Option<ArcCell>,
    pub refund_forward_gas_amount: Option<u128>,
    pub referral_address: Option<TonAddress>,
    pub referral_value: Option<u16>,
    pub deadline: Option<u64>,
    pub forward_gas_amount: Option<u128>,
    pub query_id: Option<u64>,
}

#[derive(Debug)]
pub struct SenderArguments {
    pub value: BigInt,
    pub to: TonAddress,
    //pub send_mode: Option<SendMode>,
    //pub bounce: Option<bool>,
    //pub init: Option<StateInit>,
    //pub body: Option<Cell>,
}

#[derive(Debug, Error)]
pub enum StonfiBaseRouterError {
    #[error("Tonlib error: {0}")]
    TonlibError(String),

    #[error("Ton contract error: {0}")]
    TonContractError(String),
}

impl From<TonCellError> for StonfiBaseRouterError {
    fn from(error: TonCellError) -> Self {
        Self::TonlibError(error.to_string())
    }
}

impl From<TonContractError> for StonfiBaseRouterError {
    fn from(error: TonContractError) -> Self {
        Self::TonContractError(error.to_string())
    }
}

#[derive(Debug)]
pub struct StonfiBaseRouter {
    pub address: TonAddress,
}

impl StonfiBaseRouter {
    pub fn new(address: TonAddress) -> Self {
        Self { address }
    }

    pub async fn create_swap_body(&self, body: SwapBodyParams) -> Result<Cell, StonfiBaseRouterError> {
        let excesses_address = body.excesses_address.unwrap_or(body.refund_address.clone());
        let additional_payload = CellBuilder::new()
            .store_coins(&BigUint::from(body.min_ask_amount))?
            .store_address(&body.receiver_address)?
            .store_coins(&BigUint::from(body.dex_custom_payload_forward_gas_amount.unwrap_or(0)))?
            .store_maybe_cell_ref(&body.dex_custom_payload)?
            .store_coins(&BigUint::from(body.refund_forward_gas_amount.unwrap_or(0)))?
            .store_maybe_cell_ref(&body.refund_payload)?
            .store_uint(16, &BigUint::from(body.referral_value.unwrap_or(100)))?
            .store_address(&body.referral_address.unwrap_or(TonAddress::NULL))?
            .build()?;
        let additional_payload_ref = Arc::new(additional_payload);

        Ok(CellBuilder::new()
            .store_uint(32, &BigUint::from(DEX_OP_CODES.swap))?
            .store_address(&body.ask_jetton_wallet_address)?
            .store_address(&body.refund_address)?
            .store_address(&excesses_address)?
            .store_uint(64, &BigUint::from(body.deadline.unwrap_or(TX_DEADLINE)))?
            .store_reference(&additional_payload_ref)?
            .build()?)
    }

    pub async fn get_swap_ton_to_jetton_tx_params(
        &self,
        client: Arc<TonClient>, // TODO: should be wrapper or profider that utilizes AlienProvider
        body: SwapTonToJettonTxParams,
    ) -> Result<SenderArguments, StonfiBaseRouterError> {
        // TODO: assert body.proxy_ton

        let contract_address = self.address.clone();
        let factory = TonContractFactory::builder(&client).build().await?;
        let contract = factory.get_contract(&body.ask_jetton_address);
        let ask_jetton_wallet_address = contract.get_wallet_address(&contract_address).await?;

        let forward_payload = self
            .create_swap_body(SwapBodyParams {
                ask_jetton_wallet_address,
                receiver_address: body.receiver_address.unwrap_or(body.user_wallet_address.clone()),
                min_ask_amount: body.min_ask_amount,
                refund_address: body.refund_address.unwrap_or(body.user_wallet_address.clone()),
                excesses_address: body.excesses_address,
                referral_address: body.referral_address,
                referral_value: body.referral_value,
                dex_custom_payload: body.dex_custom_payload,
                dex_custom_payload_forward_gas_amount: body.dex_custom_payload_forward_gas_amount,
                refund_payload: body.refund_payload,
                refund_forward_gas_amount: body.refund_forward_gas_amount,
                deadline: body.deadline,
            })
            .await?;

        // TODO: add gas constants
        let forward_ton_amount = body.forward_gas_amount.unwrap_or(0);
        let params = body
            .proxy_ton
            .get_ton_transfer_params(ProxyTonTransferParams {
                query_id: body.query_id.unwrap_or(0),
                ton_amount: body.offer_ammount,
                destination_address: contract_address,
                refund_address: body.user_wallet_address.clone(),
                forward_payload: forward_payload.into(),
                forward_ton_amount,
            })
            .await?;

        todo!()
    }
}
