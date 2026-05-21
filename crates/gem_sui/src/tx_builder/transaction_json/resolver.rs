use super::model::{TransactionArgument, TransactionCommand, TransactionInput};
use crate::rpc::proto::{FunctionDescriptor, OpenSignature, open_signature::Reference};
use crate::{SuiClient, SuiError, tx_builder::ResolvedObjectInput};
use futures::future::try_join_all;
use std::collections::{HashMap, HashSet};
use sui_transaction_builder::ObjectInput;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct MoveFunctionKey {
    package: String,
    module: String,
    function: String,
}

pub(super) async fn input_mutability(client: &SuiClient, commands: &[TransactionCommand]) -> Result<HashMap<usize, bool>, SuiError> {
    let keys = commands
        .iter()
        .filter_map(|command| match command {
            TransactionCommand::MoveCall { move_call } => Some(MoveFunctionKey {
                package: move_call.package.clone(),
                module: move_call.module.clone(),
                function: move_call.function.clone(),
            }),
            _ => None,
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let cache = fetch_functions(client, keys).await?;
    let mut mutable_inputs = HashMap::<usize, bool>::new();

    for command in commands {
        let TransactionCommand::MoveCall { move_call } = command else {
            continue;
        };
        let key = MoveFunctionKey {
            package: move_call.package.clone(),
            module: move_call.module.clone(),
            function: move_call.function.clone(),
        };
        let function = cache.get(&key).ok_or_else(|| SuiError::invalid_input("Missing cached Sui Move function signature"))?;
        for (argument, parameter) in move_call.arguments.iter().zip(&function.parameters) {
            if !is_mutable_parameter(parameter) {
                continue;
            }
            if let TransactionArgument::Input { input } = argument {
                mutable_inputs.insert(*input, true);
            }
        }
    }

    Ok(mutable_inputs)
}

async fn fetch_functions(client: &SuiClient, keys: Vec<MoveFunctionKey>) -> Result<HashMap<MoveFunctionKey, FunctionDescriptor>, SuiError> {
    try_join_all(keys.into_iter().map(|key| async move {
        let function = client
            .get_function(&key.package, &key.module, &key.function)
            .await
            .map_err(|err| SuiError::invalid_input(format!("Failed to fetch Sui Move function signature: {err}")))?;
        Ok((key, function))
    }))
    .await
    .map(|functions| functions.into_iter().collect())
}

pub(super) async fn object_inputs(client: &SuiClient, inputs: &[TransactionInput], input_mutability: &HashMap<usize, bool>) -> Result<HashMap<usize, ObjectInput>, SuiError> {
    let object_ids = inputs
        .iter()
        .filter_map(|input| match input {
            TransactionInput::UnresolvedObject { object } => Some(object.object_id.clone()),
            TransactionInput::Pure { .. } | TransactionInput::Object { .. } | TransactionInput::UnresolvedPure { .. } => None,
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let fetched = if object_ids.is_empty() {
        Vec::new()
    } else {
        client
            .get_multiple_objects(object_ids.clone())
            .await
            .map_err(|err| SuiError::invalid_input(format!("Failed to fetch Sui transaction objects: {err}")))?
    };
    let fetched = object_ids
        .into_iter()
        .zip(fetched)
        .map(|(object_id, object)| Ok((object_id, ResolvedObjectInput::from_rpc_object(object)?)))
        .collect::<Result<HashMap<_, _>, SuiError>>()?;

    inputs
        .iter()
        .enumerate()
        .filter_map(|(index, input)| match input {
            TransactionInput::UnresolvedObject { object } => Some(object_input_from_fetched(index, &object.object_id, &fetched, input_mutability)),
            TransactionInput::Pure { .. } | TransactionInput::Object { .. } | TransactionInput::UnresolvedPure { .. } => None,
        })
        .collect()
}

fn is_mutable_parameter(parameter: &OpenSignature) -> bool {
    parameter.reference.and_then(|reference| Reference::try_from(reference).ok()) == Some(Reference::Mutable)
}

fn object_input_from_fetched(
    index: usize,
    object_id: &str,
    fetched: &HashMap<String, ResolvedObjectInput>,
    input_mutability: &HashMap<usize, bool>,
) -> Result<(usize, ObjectInput), SuiError> {
    let object = fetched
        .get(object_id)
        .ok_or_else(|| SuiError::invalid_input(format!("Sui object was not returned by RPC: {object_id}")))?;
    Ok((index, object.input(input_mutability.get(&index).copied().unwrap_or(false))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_mutable_parameter() {
        let mut parameter = OpenSignature {
            reference: Some(Reference::Mutable as i32),
        };
        assert!(is_mutable_parameter(&parameter));

        parameter.reference = Some(Reference::Immutable as i32);
        assert!(!is_mutable_parameter(&parameter));
    }
}
