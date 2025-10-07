pub mod models;
pub use models::*;

use serde::{Serialize, de::DeserializeOwned};
use std::collections::HashSet;
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

    pub async fn sync_documents<T: Serialize + Send + Sync + Clone>(
        &self,
        index: &str,
        documents: Vec<T>,
        get_id: fn(&T) -> String,
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.add_documents(index, documents.clone()).await?;

        let db_documents_ids: HashSet<_> = documents.iter().map(get_id).collect();
        let existing_documents_ids: HashSet<_> = self.get_documents_all::<DocumentId>(index).await?.into_iter().map(|x| x.id).collect();
        let stale_ids: Vec<&str> = existing_documents_ids.difference(&db_documents_ids).map(|id| id.as_str()).collect();

        self.delete_documents(index, stale_ids).await?;

        Ok(documents.len())
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

    pub async fn setup(&self, configs: &[crate::IndexConfig], primary_key: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        for config in configs {
            self.create_index(config.name, primary_key).await?;
            self.set_filterable_attributes(config.name, config.filters.to_vec()).await?;
            self.set_sortable_attributes(config.name, config.sorts.to_vec()).await?;
            self.set_searchable_attributes(config.name, config.search_attributes.to_vec()).await?;
            self.set_ranking_rules(config.name, config.ranking_rules.to_vec()).await?;
        }
        Ok(())
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
