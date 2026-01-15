use crate::models::TransactionPayload;
use primitives::SignerError;
use serde::Deserialize;
use serde_json::Value;

use crate::r#move::{EntryFunction, encode_argument, infer_type_tags, parse_function_id, parse_type_tag};

const ENTRY_FUNCTION_PAYLOAD_TYPE: &str = "entry_function_payload";

#[derive(Clone, Debug, Deserialize)]
pub struct EntryFunctionPayload {
    #[serde(rename = "type")]
    pub payload_type: String,
    pub function: String,
    #[serde(default)]
    pub type_arguments: Vec<String>,
    #[serde(default)]
    pub arguments: Vec<Value>,
}

impl EntryFunctionPayload {
    pub fn to_transaction_payload(&self) -> TransactionPayload {
        TransactionPayload {
            function: Some(self.function.clone()),
            type_arguments: self.type_arguments.clone(),
            arguments: self.arguments.clone(),
            payload_type: self.payload_type.clone(),
        }
    }

    pub fn to_entry_function(&self, abi: Option<&[&str]>) -> Result<EntryFunction, SignerError> {
        if self.payload_type != ENTRY_FUNCTION_PAYLOAD_TYPE {
            return Err(SignerError::InvalidInput(format!("Unsupported Aptos payload type: {}", self.payload_type)));
        }

        let (module, function) = parse_function_id(&self.function)?;
        let ty_args = self.type_arguments.iter().map(|arg| parse_type_tag(arg)).collect::<Result<Vec<_>, _>>()?;

        let arg_types = match abi {
            Some(abi_types) => {
                if abi_types.len() != self.arguments.len() {
                    return Err(SignerError::InvalidInput("Aptos ABI length does not match arguments".to_string()));
                }
                abi_types.iter().map(|arg| parse_type_tag(arg)).collect::<Result<Vec<_>, _>>()?
            }
            None => infer_type_tags(&self.arguments)?,
        };

        let args = self
            .arguments
            .iter()
            .zip(arg_types.iter())
            .map(|(value, arg_type)| encode_argument(value, arg_type))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(EntryFunction {
            module,
            function,
            ty_args,
            args,
        })
    }
}
