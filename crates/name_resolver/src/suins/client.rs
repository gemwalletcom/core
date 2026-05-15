use crate::client::NameClient;
use crate::model::NameQuery;
use async_trait::async_trait;
use gem_encoding::protobuf::{decode_grpc_frame, encode_grpc_frame};
use gem_jsonrpc::grpc::{GrpcTransport, ReqwestGrpcTransport};
use primitives::NameProvider;
use primitives::chain::Chain;
use std::error::Error;

use super::proto::{decode_lookup_name_response, encode_lookup_name_request};

const PATH_LOOKUP_NAME: &str = "/sui.rpc.v2.NameService/LookupName";

#[derive(Clone, Debug)]
pub struct SuinsClient {
    api_url: String,
    transport: ReqwestGrpcTransport,
}

impl SuinsClient {
    pub fn new(api_url: String) -> Self {
        Self {
            api_url,
            transport: ReqwestGrpcTransport::new(),
        }
    }

    async fn lookup_name(&self, name: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let body = self
            .transport
            .unary(&self.api_url, PATH_LOOKUP_NAME, encode_grpc_frame(&encode_lookup_name_request(name)))
            .await
            .map_err(|error| format!("SuiNS gRPC request failed: {error}"))?;

        decode_lookup_name_response(decode_grpc_frame(&body)?)
    }
}

#[async_trait]
impl NameClient for SuinsClient {
    fn provider(&self) -> NameProvider {
        NameProvider::Suins
    }

    async fn resolve(&self, query: &NameQuery, _chain: Chain) -> Result<String, Box<dyn Error + Send + Sync>> {
        self.lookup_name(&query.domain).await
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["sui"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Sui]
    }
}

#[cfg(test)]
mod tests {
    use gem_encoding::protobuf::{encode_grpc_frame, encode_string_field};

    #[test]
    fn test_encode_grpc_message() {
        let payload = encode_string_field(1, "alpha.sui");
        let encoded = encode_grpc_frame(&payload);

        assert_eq!(encoded[0], 0);
        assert_eq!(u32::from_be_bytes(encoded[1..5].try_into().unwrap()), payload.len() as u32);
        assert_eq!(&encoded[5..], payload.as_slice());
    }
}
