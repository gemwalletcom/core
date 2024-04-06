use std::{error::Error, thread, time::Duration};

use storage::DatabaseClient;
use swap_oneinch::OneInchClient;

pub struct Client {
    client: OneInchClient,
    database: DatabaseClient,
}

impl Client {
    pub fn new(client: OneInchClient, database_url: &str) -> Self {
        Self {
            client,
            database: DatabaseClient::new(database_url),
        }
    }

    pub async fn update_swap_tokenlist(&mut self) -> Result<usize, Box<dyn Error>> {
        let chains = self.client.chains();

        for chain in chains {
            let tokenlist = self.client.get_tokenlist(chain.network_id()).await?;
            let asset_ids = self
                .client
                .get_asset_ids_for_tokenlist(chain, tokenlist)
                .into_iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>();

            println!("update swap tokenlist: {chain} {:?}", asset_ids.len());

            self.database.set_swap_enabled(asset_ids)?;

            thread::sleep(Duration::from_secs(1));
        }

        Ok(0)
    }
}
