use api_connector::AssetsClient;
use std::error::Error;
use storage::{database::DatabaseClient, models::Asset};

pub struct Client {
    database: DatabaseClient,
    assets_client: AssetsClient,
}

impl Client {
    pub fn new(database_url: &str, assets_client: AssetsClient) -> Self {
        let database = DatabaseClient::new(database_url);
        Self { database, assets_client }
    }

    pub async fn update(&mut self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let lists = self.database.get_tokenlists()?;
        for list in &lists {
            let tokenlist = self.assets_client.get_tokenlist(list.chain.as_ref()).await?;
            let _ = self.database.set_tokenlist_version(list.clone().chain, tokenlist.version);

            let assets = tokenlist.assets.into_iter().map(|x| Asset::from_primitive(x.to_asset())).collect();
            let _ = self.database.add_assets(assets);
        }
        Ok(lists.len())
    }
}
