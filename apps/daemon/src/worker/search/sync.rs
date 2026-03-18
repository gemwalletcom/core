use chrono::{NaiveDateTime, Utc};
use primitives::ConfigKey;
use search_index::SearchIndexClient;
use serde::Serialize;
use std::error::Error;
use std::fmt;
use storage::{ConfigCacher, Database};

#[derive(Clone, Copy)]
pub enum SearchSyncAction {
    ReplaceIndex,
    IncrementalUpdate,
}

impl fmt::Display for SearchSyncAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SearchSyncAction::ReplaceIndex => write!(f, "replace"),
            SearchSyncAction::IncrementalUpdate => write!(f, "incremental"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct SearchSyncResult {
    action: SearchSyncAction,
    indexed_documents: usize,
}

impl fmt::Debug for SearchSyncResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} indexed_documents={}", self.action, self.indexed_documents)
    }
}

pub struct SearchSyncClient {
    config: ConfigCacher,
    search_index: SearchIndexClient,
}

impl SearchSyncClient {
    pub fn new(database: Database, search_index: &SearchIndexClient) -> Self {
        Self {
            config: ConfigCacher::new(database),
            search_index: search_index.clone(),
        }
    }

    pub fn for_key(&self, key: ConfigKey) -> Result<IndexSync<'_>, Box<dyn Error + Send + Sync>> {
        Ok(IndexSync {
            client: self,
            key: key.clone(),
            last_updated_at: self.config.get_datetime(key)?,
            now: Utc::now().naive_utc(),
        })
    }
}

pub struct IndexSync<'a> {
    client: &'a SearchSyncClient,
    key: ConfigKey,
    last_updated_at: NaiveDateTime,
    now: NaiveDateTime,
}

impl IndexSync<'_> {
    pub fn action(&self) -> SearchSyncAction {
        if self.should_replace_index() {
            SearchSyncAction::ReplaceIndex
        } else {
            SearchSyncAction::IncrementalUpdate
        }
    }

    pub fn should_replace_index(&self) -> bool {
        self.last_updated_at.and_utc().timestamp() == 0
    }

    pub fn last_updated_at(&self) -> NaiveDateTime {
        self.last_updated_at
    }

    pub async fn write<T: Serialize + Send + Sync>(self, index: &str, documents: Vec<T>) -> Result<SearchSyncResult, Box<dyn Error + Send + Sync>> {
        let action = self.action();
        let indexed_documents = if self.should_replace_index() {
            self.client.search_index.replace_documents(index, documents).await?
        } else {
            self.client.search_index.index_documents(index, documents).await?
        };

        self.client.config.set_datetime(self.key, self.now)?;
        Ok(SearchSyncResult { action, indexed_documents })
    }
}
