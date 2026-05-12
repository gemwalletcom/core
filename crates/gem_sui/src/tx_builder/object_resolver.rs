use crate::{
    SuiClient, SuiError,
    jsonrpc::{DataObject, ObjectDataOptions, SuiRpc},
    models::ResultData,
};
use gem_client::Client;
use std::collections::{BTreeSet, HashMap};
use sui_transaction_builder::{Argument, ObjectInput, TransactionBuilder};

pub struct ObjectResolver {
    objects: HashMap<String, DataObject<()>>,
}

impl ObjectResolver {
    pub async fn prefetch<C: Client + Clone>(client: &SuiClient<C>, object_ids: Vec<String>) -> Result<Self, SuiError> {
        let object_ids = object_ids.into_iter().collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>();
        if object_ids.is_empty() {
            return Ok(Self { objects: HashMap::new() });
        }

        let options = ObjectDataOptions {
            show_type: false,
            show_owner: true,
            show_display: false,
            show_content: false,
            show_bcs: false,
        };
        let response: Vec<ResultData<DataObject<()>>> = client
            .rpc_call(SuiRpc::GetMultipleObjects(object_ids.clone(), Some(options)))
            .await
            .map_err(|err| SuiError::invalid_input(err.to_string()))?;
        if response.len() != object_ids.len() {
            return Err(SuiError::invalid_input(format!(
                "Sui object response count mismatch: requested {}, received {}",
                object_ids.len(),
                response.len()
            )));
        }

        Ok(Self {
            objects: object_ids.into_iter().zip(response).map(|(object_id, response)| (object_id, response.data)).collect(),
        })
    }

    pub fn shared_object_input(&self, object_id: &str, mutable: bool) -> Result<ObjectInput, SuiError> {
        let data = self.object_data(object_id)?;
        let initial_shared_version = data
            .initial_shared_version()
            .ok_or_else(|| SuiError::invalid_input(format!("Object is not shared: {object_id}")))?;
        Ok(ObjectInput::shared(data.object_id, initial_shared_version, mutable))
    }

    pub fn shared_object(&self, txb: &mut TransactionBuilder, object_id: &str, mutable: bool) -> Result<Argument, SuiError> {
        Ok(txb.object(self.shared_object_input(object_id, mutable)?))
    }

    fn object_data(&self, object_id: &str) -> Result<DataObject<()>, SuiError> {
        self.objects
            .get(object_id)
            .cloned()
            .ok_or_else(|| SuiError::invalid_input(format!("Sui object was not prefetched: {object_id}")))
    }
}
