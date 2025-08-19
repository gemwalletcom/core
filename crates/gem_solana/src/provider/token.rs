use async_trait::async_trait;
use chain_traits::ChainToken;
use std::error::Error;

use gem_client::Client;
use primitives::Asset;

use crate::{rpc::client::SolanaClient, rpc::mapper::SolanaMapper};

#[async_trait]
impl<C: Client + Clone> ChainToken for SolanaClient<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Sync + Send>> {
        let token_info_result = self.get_token_mint_info(&token_id).await?;
        let token_info = token_info_result.info();
        
        if let Some(extensions) = &token_info.extensions {
            if let Some(ext) = extensions.iter().find(|ext| matches!(ext, crate::model::Extension::TokenMetadata(_))) {
                if let crate::model::Extension::TokenMetadata(_token_metadata) = ext {
                    return SolanaMapper::map_token_data_spl_token_2022(self.get_chain(), token_id, &token_info);
                }
            }
        }
        
        let metadata = self.get_metaplex_metadata(&token_id).await?;
        SolanaMapper::map_token_data(self.get_chain(), token_id, &token_info, &metadata)
    }

    fn get_is_token_address(&self, token_id: &str) -> bool {
        token_id.len() >= 40 && token_id.len() <= 60 && bs58::decode(token_id).into_vec().is_ok()
    }
}


