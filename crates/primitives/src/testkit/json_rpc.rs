use crate::JsonRpcResult;

pub fn load_json_rpc_result<T>(json: &str) -> T
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_str::<JsonRpcResult<T>>(json).unwrap().result
}
