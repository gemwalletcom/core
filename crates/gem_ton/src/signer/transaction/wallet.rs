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
    code: CellArc,
    data: CellArc,
}

impl StateInit {
    fn to_cell(&self) -> Result<Cell, SignerError> {
        let mut builder = CellBuilder::new();
        builder
            .store_bit(false)?
            .store_bit(false)?
            .store_bit(true)?
            .store_bit(true)?
            .store_bit(false)?
            .store_reference(&self.code)?
            .store_reference(&self.data)?;
        builder.build()
    }
}

pub struct WalletV4R2 {
    public_key: [u8; 32],
    address: Address,
}

impl WalletV4R2 {
    pub fn new(public_key: [u8; 32]) -> Result<Self, SignerError> {
        let state_init = Self::state_init(&public_key)?;
        Ok(Self {
            public_key,
            address: Address::new(BASE_WORKCHAIN, state_init.to_cell()?.hash),
        })
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
            builder.store_bit(true)?.store_bit(true)?.store_child(Self::state_init(&self.public_key)?.to_cell()?)?;
        } else {
            builder.store_bit(false)?;
        }
        builder.store_bit(true)?.store_child(signed_body)?;
        builder.build()
    }

    fn state_init(public_key: &[u8; 32]) -> Result<StateInit, SignerError> {
        let mut data = CellBuilder::new();
        data.store_u32(32, 0)?.store_i32(32, DEFAULT_WALLET_ID)?.store_slice(public_key)?.store_bit(false)?;

        Ok(StateInit {
            code: BagOfCells::parse_base64(WALLET_V4R2_CODE_BOC.trim())?.get_single_root()?.clone(),
            data: data.build()?.into_arc(),
        })
    }
}
