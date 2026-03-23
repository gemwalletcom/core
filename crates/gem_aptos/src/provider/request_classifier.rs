use chain_traits::ChainRequestClassifier;
use primitives::{ChainRequest, ChainRequestType};

use crate::provider::BroadcastProvider;

impl ChainRequestClassifier for BroadcastProvider {
    fn classify_request(&self, request: ChainRequest<'_>) -> ChainRequestType {
        if request.is_http_post_path("/v1/transactions") {
            ChainRequestType::Broadcast
        } else {
            ChainRequestType::Unknown
        }
    }
}
