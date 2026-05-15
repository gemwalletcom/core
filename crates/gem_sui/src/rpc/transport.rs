use std::sync::Arc;

use gem_jsonrpc::grpc::GrpcTransport;
#[cfg(feature = "reqwest")]
use gem_jsonrpc::grpc::ReqwestGrpcTransport;

#[cfg(feature = "reqwest")]
pub(super) fn default_transport() -> Option<Arc<dyn GrpcTransport>> {
    Some(Arc::new(ReqwestGrpcTransport::new()))
}

#[cfg(not(feature = "reqwest"))]
pub(super) fn default_transport() -> Option<Arc<dyn GrpcTransport>> {
    None
}
