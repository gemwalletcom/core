use crate::rpc::proto::{Object, OwnerKind};
use crate::{SuiClient, SuiError};
use std::{
    collections::{BTreeSet, HashMap},
    str::FromStr,
};
use sui_transaction_builder::{Argument, ObjectInput, TransactionBuilder};
use sui_types::{Address, Digest};

#[derive(Clone, Debug)]
pub struct ResolvedObjectInput {
    object_id: Address,
    owner: ResolvedObjectOwner,
}

#[derive(Clone, Debug)]
enum ResolvedObjectOwner {
    Shared { initial_shared_version: u64 },
    Owned { version: u64, digest: Digest },
    Immutable { version: u64, digest: Digest },
}

impl ResolvedObjectInput {
    pub async fn get_multiple(client: &SuiClient, object_ids: Vec<String>) -> Result<Vec<Self>, SuiError> {
        client
            .get_multiple_objects(object_ids)
            .await
            .map_err(SuiError::from_display)?
            .into_iter()
            .map(Self::from_rpc_object)
            .collect()
    }

    pub(crate) fn from_rpc_object(object: Object) -> Result<Self, SuiError> {
        let object_id = object.object_id.ok_or_else(|| SuiError::invalid_input("missing Sui object id"))?;
        let owner = object.owner.ok_or_else(|| SuiError::invalid_input(format!("Sui object owner is missing: {object_id}")))?;
        let owner_kind = owner.kind();
        let object_id = Address::from_str(&object_id).map_err(|err| SuiError::invalid_input(format!("Invalid Sui object id {object_id}: {err}")))?;

        let owner = match owner_kind {
            OwnerKind::Shared => ResolvedObjectOwner::Shared {
                initial_shared_version: owner
                    .version
                    .ok_or_else(|| SuiError::invalid_input(format!("Sui shared object version is missing: {object_id}")))?,
            },
            OwnerKind::Address | OwnerKind::Object => ResolvedObjectOwner::Owned {
                version: object
                    .version
                    .ok_or_else(|| SuiError::invalid_input(format!("Sui object version is missing: {object_id}")))?,
                digest: digest(object.digest, object_id)?,
            },
            OwnerKind::Immutable => ResolvedObjectOwner::Immutable {
                version: object
                    .version
                    .ok_or_else(|| SuiError::invalid_input(format!("Sui object version is missing: {object_id}")))?,
                digest: digest(object.digest, object_id)?,
            },
            OwnerKind::Unknown | OwnerKind::ConsensusAddress => {
                return Err(SuiError::invalid_input(format!("Unsupported Sui object owner kind for {object_id}: {owner_kind:?}")));
            }
        };
        Ok(Self { object_id, owner })
    }

    pub fn input(&self, mutable: bool) -> ObjectInput {
        match &self.owner {
            ResolvedObjectOwner::Shared { initial_shared_version } => ObjectInput::shared(self.object_id, *initial_shared_version, mutable),
            ResolvedObjectOwner::Owned { version, digest } => ObjectInput::owned(self.object_id, *version, *digest),
            ResolvedObjectOwner::Immutable { version, digest } => ObjectInput::immutable(self.object_id, *version, *digest),
        }
    }
}

fn digest(value: Option<String>, object_id: Address) -> Result<Digest, SuiError> {
    let value = value.ok_or_else(|| SuiError::invalid_input(format!("Sui object digest is missing: {object_id}")))?;
    Digest::from_str(&value).map_err(|err| SuiError::invalid_input(format!("Invalid Sui object digest for {object_id}: {err}")))
}

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
            client.get_multiple_objects(missing).await.map_err(SuiError::from_display)?
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
