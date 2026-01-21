use cacher::CacherClient;
use std::error::Error;
use storage::{Database, DevicesRepository};

const DEVICE_ROW_ID_CACHE_TTL: u64 = 86400;

#[derive(Clone)]
pub struct DeviceCacher {
    database: Database,
    cacher: CacherClient,
}

impl DeviceCacher {
    pub fn new(database: Database, cacher: CacherClient) -> Self {
        Self { database, cacher }
    }

    pub async fn get_device_row_id(&self, device_id: &str) -> Result<i32, Box<dyn Error + Send + Sync>> {
        let cache_key = format!("device_row_id:{}", device_id);
        let database = self.database.clone();
        let device_id_owned = device_id.to_string();

        self.cacher
            .get_or_set_value(
                &cache_key,
                || async move { Ok(database.devices()?.get_device_row_id(&device_id_owned)?) },
                Some(DEVICE_ROW_ID_CACHE_TTL),
            )
            .await
    }
}
