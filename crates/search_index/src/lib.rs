pub mod model;
pub use model::*;

use serde::{de::DeserializeOwned, Serialize};
use std::error::Error;

use meilisearch_sdk::{client::*, documents::DocumentsQuery, task_info::TaskInfo};

#[derive(Debug, Clone)]
pub struct SearchIndexClient {
    client: Client,
}

const DOCUMENTS_FETCH_LIMIT: usize = 1000;

impl SearchIndexClient {
    pub fn new(url: &str, api_key: &str) -> Self {
        let client = Client::new(url.to_string(), Some(api_key)).unwrap();
        Self { client }
    }

    pub async fn create_index(&self, name: &str, primary_key: &str) -> Result<TaskInfo, Box<dyn Error + Send + Sync>> {
        Ok(self.client.create_index(name, Some(primary_key)).await?)
    }

    pub async fn get_documents_all<T: DeserializeOwned + Send + Sync + 'static>(&self, index: &str) -> Result<Vec<T>, Box<dyn Error + Send + Sync>> {
        let index_ref = self.client.index(index);
        let mut all_documents = Vec::new();
        let mut offset = 0;
        let limit = DOCUMENTS_FETCH_LIMIT;

        loop {
            let mut query = DocumentsQuery::new(&index_ref);
            query.with_limit(limit).with_offset(offset);
            let documents = index_ref.get_documents_with(&query).await?.results;

            if documents.is_empty() || documents.len() < limit {
                all_documents.extend(documents);
                break;
            }

            all_documents.extend(documents);
            offset += limit;
        }

        Ok(all_documents)
    }

    pub async fn add_documents<T: Serialize + Send + Sync>(&self, index: &str, documents: Vec<T>) -> Result<TaskInfo, Box<dyn Error + Send + Sync>> {
        Ok(self.client.index(index).add_documents(&documents, None).await?)
    }

    pub async fn delete_documents(&self, index: &str, ids: Vec<&str>) -> Result<TaskInfo, Box<dyn Error + Send + Sync>> {
        Ok(self.client.index(index).delete_documents(&ids).await?)
    }

    pub async fn delete_all_documents(&self, index: &str) -> Result<TaskInfo, Box<dyn Error + Send + Sync>> {
        Ok(self.client.index(index).delete_all_documents().await?)
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
