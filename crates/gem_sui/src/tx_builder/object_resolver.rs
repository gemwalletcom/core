use crate::rpc::proto::OwnerKind;
use crate::{SuiClient, SuiError};
use std::{
    collections::{BTreeSet, HashMap},
    str::FromStr,
};
use sui_transaction_builder::{Argument, ObjectInput, TransactionBuilder};
use sui_types::Address;

pub struct ObjectResolver {
    shared_versions: HashMap<String, u64>,
}

impl ObjectResolver {
    pub async fn prefetch(client: &SuiClient, object_ids: Vec<String>, pinned: &HashMap<String, u64>) -> Result<Self, SuiError> {
        let unique_ids: Vec<String> = object_ids.into_iter().collect::<BTreeSet<_>>().into_iter().collect();
        let missing: Vec<String> = unique_ids.iter().filter(|id| !pinned.contains_key(*id)).cloned().collect();

        let fetched = if missing.is_empty() {
            Vec::new()
        } else {
            client.get_multiple_objects(missing).await.map_err(|err| SuiError::invalid_input(err.to_string()))?
        };

        let mut shared_versions: HashMap<String, u64> = fetched
            .into_iter()
            .filter_map(|object| {
                let id = object.object_id?;
                object.owner.and_then(|owner| match owner.kind() {
                    OwnerKind::Shared => owner.version.map(|version| (id, version)),
                    _ => None,
                })
            })
            .collect();
        for id in &unique_ids {
            if let Some(&version) = pinned.get(id) {
                shared_versions.insert(id.clone(), version);
            }
        }
        Ok(Self { shared_versions })
    }

    pub fn initial_shared_version(&self, object_id: &str) -> Option<u64> {
        self.shared_versions.get(object_id).copied()
    }

    pub fn shared_object_input(&self, object_id: &str, mutable: bool) -> Result<ObjectInput, SuiError> {
        let version = self
            .shared_versions
            .get(object_id)
            .copied()
            .ok_or_else(|| SuiError::invalid_input(format!("Sui shared object was not prefetched: {object_id}")))?;
        let address = Address::from_str(object_id).map_err(|err| SuiError::invalid_input(format!("Invalid Sui address {object_id}: {err}")))?;
        Ok(ObjectInput::shared(address, version, mutable))
    }

    pub fn shared_object(&self, txb: &mut TransactionBuilder, object_id: &str, mutable: bool) -> Result<Argument, SuiError> {
        Ok(txb.object(self.shared_object_input(object_id, mutable)?))
    }
}
