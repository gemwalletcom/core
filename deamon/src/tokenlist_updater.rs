use std::error::Error;
use storage::database::DatabaseClient;
use api_connector::AssetsClient;

pub struct Client<'a> {
    database: DatabaseClient,
    assets_client: &'a AssetsClient,
}

impl<'a> Client<'a> {
    pub fn new(database_url: &str, assets_client: &'a AssetsClient) -> Self {
        let database = DatabaseClient::new(database_url);
        Self {
            database,
            assets_client,
        }
    }

    pub async fn update_versions(&mut self) -> Result<usize, Box<dyn Error>> {
        let lists = self.database.get_tokenlists()?;
        for list in &lists {
            let version = self.assets_client.get_tokenlist(list.chain.as_str()).await?.version;
            let _ = self.database.set_tokenlist_version(list.clone().chain, version);
        }
        Ok(lists.len())
    }
}
