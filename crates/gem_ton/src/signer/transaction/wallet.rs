use num_bigint::BigUint;
use primitives::SignerError;

use super::message::InternalMessage;
use crate::address::Address;
use crate::signer::cells::{BagOfCells, Cell, CellArc, CellBuilder};

const BASE_WORKCHAIN: i32 = 0;
const DEFAULT_WALLET_ID: i32 = 0x29a9a317;
const WALLET_V4R2_CODE_BOC: &str = include_str!("wallet_v4r2_code.boc.b64");

#[derive(Clone)]
struct StateInit {
    code: Option<CellArc>,
    data: Option<CellArc>,
}

impl StateInit {
    fn to_cell(&self) -> Result<Cell, SignerError> {
        let mut builder = CellBuilder::new();
        builder
            .store_bit(false)?
            .store_bit(false)?
            .store_bit(self.code.is_some())?
            .store_bit(self.data.is_some())?
            .store_bit(false)?;
        if let Some(code) = &self.code {
            builder.store_reference(code)?;
        }
        if let Some(data) = &self.data {
            builder.store_reference(data)?;
        }
        builder.build()
    }
}

pub(super) struct WalletV4R2 {
    public_key: [u8; 32],
    address: Address,
}

impl WalletV4R2 {
    pub(super) fn new(public_key: [u8; 32]) -> Result<Self, SignerError> {
        let state_init = wallet_state_init(&public_key)?;
        Ok(Self {
            public_key,
            address: Address::new(BASE_WORKCHAIN, state_init.to_cell()?.cell_hash()),
        })
    }

    #[cfg(test)]
    pub(super) fn address(&self) -> &Address {
        &self.address
    }

    pub(super) fn build_external_body(&self, expire_at: u32, sequence: u32, messages: &[InternalMessage]) -> Result<Cell, SignerError> {
        let mut builder = CellBuilder::new();
        builder
            .store_i32(32, DEFAULT_WALLET_ID)?
            .store_u32(32, expire_at)?
            .store_u32(32, sequence)?
            .store_u8(8, 0)?;
        for message in messages {
            builder.store_u8(8, message.mode)?.store_child(message.message.clone())?;
        }
        builder.build()
    }

    pub(super) fn build_transaction(&self, include_state_init: bool, signed_body: Cell) -> Result<Cell, SignerError> {
        let mut builder = CellBuilder::new();
        builder
            .store_u8(2, 0b10)?
            .store_null_address()?
            .store_address(&self.address)?
            .store_coins(&BigUint::from(0u8))?;

        if include_state_init {
            builder.store_bit(true)?.store_bit(true)?.store_child(wallet_state_init(&self.public_key)?.to_cell()?)?;
        } else {
            builder.store_bit(false)?;
        }
        builder.store_bit(true)?.store_child(signed_body)?;
        builder.build()
    }
}

fn wallet_state_init(public_key: &[u8; 32]) -> Result<StateInit, SignerError> {
    let mut data = CellBuilder::new();
    data.store_u32(32, 0)?.store_i32(32, DEFAULT_WALLET_ID)?.store_slice(public_key)?.store_bit(false)?;

    let code = BagOfCells::parse_base64(WALLET_V4R2_CODE_BOC)?.single_root()?.clone();
    Ok(StateInit {
        code: Some(code),
        data: Some(data.build()?.into_arc()),
    })
}

pub(super) fn build_signed_message(signature: &[u8; 64], external_body: &Cell) -> Result<Cell, SignerError> {
    let mut builder = CellBuilder::new();
    builder.store_slice(signature)?.store_cell(external_body)?;
    builder.build()
}
