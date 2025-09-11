use std::error::Error;

use async_trait::async_trait;
use gem_tracing::info_with_context;
use streamer::consumer::MessageConsumer;
use streamer::SupportWebhookPayload;

pub struct SupportWebhookConsumer;

impl SupportWebhookConsumer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl MessageConsumer<SupportWebhookPayload, bool> for SupportWebhookConsumer {
    async fn should_process(&mut self, _payload: SupportWebhookPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&mut self, payload: SupportWebhookPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        info_with_context("support webhook received", &[("data", &format!("{}", payload.data))]);

        Ok(true)
    }
}
