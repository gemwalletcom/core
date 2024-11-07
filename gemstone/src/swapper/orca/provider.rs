use super::whirlpool::get_whirlpool_address;
use super::{models::*, ORCA_NAME, WHIRLPOOL_CONFIG, WHIRLPOOL_PROGRAM};
use crate::network::JsonRpcResult;
use crate::{
    network::{jsonrpc::jsonrpc_call, AlienProvider},
    swapper::{models::*, GemSwapProvider, SwapperError},
};
use async_trait::async_trait;
use gem_solana::{
    jsonrpc::{Filter, Memcmp, SolanaRpc, ENCODING_BASE58},
    pubkey::Pubkey,
    WSOL_TOKEN_ADDRESS,
};
use primitives::{AssetId, Chain};
use std::{str::FromStr, sync::Arc};

#[derive(Debug, Default)]
pub struct Orca {}

#[async_trait]
impl GemSwapProvider for Orca {
    fn name(&self) -> &'static str {
        ORCA_NAME
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        if request.from_asset.chain != Chain::Solana || request.to_asset.chain != Chain::Solana {
            return Err(SwapperError::NotSupportedChain);
        }
        let from_asset = Self::get_asset_address(request.from_asset.clone())?;
        let to_asset = Self::get_asset_address(request.to_asset.clone())?;
        let config = Pubkey::from_str(WHIRLPOOL_CONFIG).unwrap();
        let tick_space = 100;
        let whirlpool_address = get_whirlpool_address(&config, &from_asset, &to_asset, tick_space);

        println!("whirlpool_address: {:?}", whirlpool_address);

        let call = SolanaRpc::GetProgramAccounts(WHIRLPOOL_PROGRAM.into(), Self::get_program_filters());
        let response: JsonRpcResult<Vec<ProgramAccount>> = jsonrpc_call(&call, provider, request.from_asset.chain).await?;

        println!("response: {:?}", response);

        Ok(SwapQuote {
            chain_type: request.from_asset.chain.chain_type(),
            from_value: request.value.clone(),
            to_value: "0".into(),
            provider: SwapProviderData {
                name: self.name().into(),
                routes: vec![SwapRoute {
                    route_type: "whirlpool".into(),
                    input: from_asset.to_string(),
                    output: to_asset.to_string(),
                    fee_tier: tick_space.to_string(),
                    gas_estimate: None,
                }],
            },
            approval: ApprovalType::None,
            request: request.clone(),
        })
    }

    async fn fetch_quote_data(&self, _quote: &SwapQuote, _provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        todo!()
    }
}

impl Orca {
    pub fn get_asset_address(asset_id: AssetId) -> Result<Pubkey, SwapperError> {
        let address = match asset_id.token_id {
            Some(token_id) => token_id,
            None => WSOL_TOKEN_ADDRESS.to_string(),
        };
        Pubkey::from_str(&address).map_err(|_| SwapperError::InvalidAddress { address })
    }

    pub fn get_program_filters() -> Vec<Filter> {
        vec![
            Filter {
                memcmp: Memcmp {
                    offset: 0,
                    bytes: "AR8t9QRDQXa".into(),
                    encoding: ENCODING_BASE58.into(),
                },
            },
            Filter {
                memcmp: Memcmp {
                    offset: 8,
                    bytes: WHIRLPOOL_CONFIG.into(),
                    encoding: ENCODING_BASE58.into(),
                },
            },
        ]
    }
}
