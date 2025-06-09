use async_trait::async_trait;
use std::{
    fmt::{self, Debug},
    time::Duration,
};

use crate::{
    provider::{AlienProvider, Data},
    target::AlienTarget,
    AlienError,
};
use primitives::Chain;

pub struct MockFn(pub Box<dyn Fn(AlienTarget) -> String + Send + Sync>);

impl fmt::Debug for MockFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("MockFn").finish()
    }
}

#[derive(Debug)]
pub struct AlienProviderMock {
    pub response: MockFn,
    pub timeout: Duration,
}

#[async_trait]
impl AlienProvider for AlienProviderMock {
    async fn request(&self, target: AlienTarget) -> Result<Data, AlienError> {
        let responses = self.batch_request(vec![target]).await;
        responses.map(|responses| responses.first().unwrap().clone())
    }

    async fn batch_request(&self, targets: Vec<AlienTarget>) -> Result<Vec<Data>, AlienError> {
        targets.iter().map(|target| Ok(self.response.0(target.clone()).into_bytes())).collect()
    }

    fn get_endpoint(&self, _chain: Chain) -> Result<String, AlienError> {
        Ok(String::from("http://localhost:8080"))
    }
}
