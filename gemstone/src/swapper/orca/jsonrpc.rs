use crate::network::jsonrpc::{JsonRpcRequest, JsonRpcRequestConvert};
use gem_solana::jsonrpc::{Configuration, SolanaRpc};
use serde_json::Value;

impl JsonRpcRequestConvert for SolanaRpc {
    fn to_req(&self, id: u64) -> JsonRpcRequest {
        let val = self;
        let method = val.to_string();
        let default_config = Configuration::default();

        let params: Vec<Value> = match val {
            SolanaRpc::GetProgramAccounts(program, filters) => vec![
                Value::String(program.into()),
                serde_json::to_value(Configuration::new(filters.to_vec())).unwrap(),
            ],
            SolanaRpc::GetAccountInfo(program) => vec![Value::String(program.into()), serde_json::to_value(default_config).unwrap()],
            SolanaRpc::GetMultipleAccounts(accounts) => vec![
                Value::Array(accounts.iter().map(|x| serde_json::to_value(x).unwrap()).collect()),
                serde_json::to_value(default_config).unwrap(),
            ],
            SolanaRpc::GetEpochInfo | SolanaRpc::GetLatestBlockhash => vec![],
        };

        JsonRpcRequest::new(id, &method, params)
    }
}
