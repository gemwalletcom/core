use std::pin::Pin;

use bytes::Bytes;
use futures::Future;
use http_body_util::Full;
use hyper::{body::Incoming as IncomingBody, service::Service, Request, Response};

use crate::metrics::Metrics;

#[derive(Debug, Clone)]
pub struct MetricsService {
    pub metrics: Metrics,
}

impl Service<Request<IncomingBody>> for MetricsService {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, _req: Request<IncomingBody>) -> Self::Future {
        let res = Ok(Response::builder().body(Full::new(Bytes::from(self.metrics.get_metrics()))).unwrap());

        Box::pin(async { res })
    }
}
