use chain_traits::ChainRequestClassifier;
use primitives::{ChainRequest, ChainRequestType};

use crate::provider::BroadcastProvider;

impl ChainRequestClassifier for BroadcastProvider {
    fn classify_request(&self, request: ChainRequest<'_>) -> ChainRequestType {
        if !request.is_http_post_path("/") {
            return ChainRequestType::Unknown;
        }

        let Some(body) = request.body_utf8() else {
            return ChainRequestType::Unknown;
        };

        if body.contains("\"operationName\":\"SubmitTransaction\"") || body.contains("submitTransaction") {
            ChainRequestType::Broadcast
        } else {
            ChainRequestType::Unknown
        }
    }
}
