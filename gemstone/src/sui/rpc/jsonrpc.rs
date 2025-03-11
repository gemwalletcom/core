use crate::network::jsonrpc::{JsonRpcRequest, JsonRpcRequestConvert};
use gem_sui::jsonrpc::SuiRpc;
use serde_json::Value;

impl JsonRpcRequestConvert for SuiRpc {
    fn to_req(&self, id: u64) -> JsonRpcRequest {
        let val = self;
        let method = val.to_string();

        let params: Vec<Value> = match val {
            SuiRpc::GetObject(object_id, options) => {
                let mut array = vec![Value::String(object_id.into())];
                if let Some(data) = options {
                    let object = serde_json::to_value(data).unwrap();
                    array.push(object);
                }
                array
            }
            SuiRpc::GetMultipleObjects(object_ids, options) => {
                let mut array = vec![Value::Array(object_ids.iter().map(|x| Value::String(x.into())).collect())];
                if let Some(data) = options {
                    let object = serde_json::to_value(data).unwrap();
                    array.push(object);
                }
                array
            }
            SuiRpc::NormalizedMoveFunction(params) => params.iter().map(|x| Value::String(x.into())).collect(),
            SuiRpc::InspectTransactionBlock(sender, tx_bytes) => {
                vec![
                    Value::String(sender.into()),
                    Value::String(tx_bytes.into()),
                    Value::Null, // gas_price
                ]
            }
            SuiRpc::GetAllCoins { owner } => {
                vec![Value::String(owner.into())]
            }
            SuiRpc::GetGasPrice => {
                vec![]
            }
        };

        JsonRpcRequest::new(id, &method, params)
    }
}
