use crate::{
    network::{jsonrpc::batch_jsonrpc_call, AlienProvider, JsonRpcRequest, JsonRpcRequestConvert, JsonRpcResponse, JsonRpcResult},
    swapper::{
        approval::{check_approval, CheckApprovalType},
        models::*,
        slippage::apply_slippage_in_bp,
        GemSwapProvider, SwapperError,
    },
};
use gem_evm::{
    across::deployment::AcrossDeployment,
    address::EthereumAddress,
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
};
use primitives::{AssetId, Chain, EVMChain};

use alloy_core::{
    primitives::{
        hex::{decode as HexDecode, encode_prefixed as HexEncode},
        Address, Bytes, U256,
    },
    sol_types::SolCall,
};
use async_trait::async_trait;
use serde_json::Value;
use std::{
    fmt::Debug,
    str::FromStr,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Default)]
pub struct Across {}

#[async_trait]
impl GemSwapProvider for Across {
    fn provider(&self) -> SwapProvider {
        SwapProvider::Across
    }

    fn supported_chains(&self) -> Vec<Chain> {
        AcrossDeployment::deployed_chains()
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        todo!()
    }
    async fn fetch_quote_data(&self, quote: &SwapQuote, provider: Arc<dyn AlienProvider>, data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        todo!()
    }
    async fn get_transaction_status(&self, _chain: Chain, _transaction_hash: &str, _provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError> {
        todo!()
    }
}
