pub mod model;
pub use model::*;

use serde::{de::DeserializeOwned, Serialize};
use std::error::Error;

use meilisearch_sdk::{client::*, task_info::TaskInfo};

#[derive(Debug, Clone)]
pub struct SearchIndexClient {
    client: Client,
}

impl SearchIndexClient {
    pub fn new(url: &str, api_key: &str) -> Self {
        let client = Client::new(url.to_string(), Some(api_key)).unwrap();
        Self { client }
    }

    pub async fn create_index(&self, name: &str, primary_key: &str) -> Result<TaskInfo, Box<dyn Error + Send + Sync>> {
        Ok(self.client.create_index(name, Some(primary_key)).await?)
    }

    pub async fn add_documents<T: Serialize + Send + Sync>(&self, index: &str, documents: Vec<T>) -> Result<TaskInfo, Box<dyn Error + Send + Sync>> {
        Ok(self.client.index(index).add_documents(&documents, None).await?)
    }

    pub async fn set_filterable_attributes(&self, index: &str, attributes: Vec<&str>) -> Result<TaskInfo, Box<dyn Error + Send + Sync>> {
        Ok(self.client.index(index).set_filterable_attributes(attributes).await?)
    }

    pub async fn set_sortable_attributes(&self, index: &str, attributes: Vec<&str>) -> Result<TaskInfo, Box<dyn Error + Send + Sync>> {
        Ok(self.client.index(index).set_sortable_attributes(attributes).await?)
    }

    pub async fn set_searchable_attributes(&self, index: &str, attributes: Vec<&str>) -> Result<TaskInfo, Box<dyn Error + Send + Sync>> {
        Ok(self.client.index(index).set_searchable_attributes(attributes).await?)
    }

    pub async fn set_ranking_rules(&self, index: &str, attributes: Vec<&str>) -> Result<TaskInfo, Box<dyn Error + Send + Sync>> {
        Ok(self.client.index(index).set_ranking_rules(attributes).await?)
    }

    // search

    pub async fn search<T: DeserializeOwned + Send + Sync + 'static>(
        &self,
        index: &str,
        query: &str,
        filter: &str,
        sort: &[&str],
        limit: usize,
        offset: usize,
    ) -> Result<Vec<T>, Box<dyn Error + Send + Sync>> {
        let results = self
            .client
            .index(index)
            .search()
            .with_query(query)
            .with_filter(filter)
            .with_sort(sort)
            .with_limit(limit)
            .with_offset(offset)
            .execute::<T>()
            .await?;

        Ok(results.hits.into_iter().map(|x| x.result).collect())
    }
}
