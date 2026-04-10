use crate::JsonRpcResult;
use serde::de::DeserializeOwned;

pub fn load_json<T>(json: &str) -> T
where
    T: DeserializeOwned,
{
    serde_json::from_str(json).unwrap()
}

pub fn load_testdata<T>(name: &str) -> T
where
    T: DeserializeOwned,
{
    load_json(&std::fs::read_to_string(std::env::current_dir().unwrap().join("testdata").join(name)).unwrap())
}

pub fn load_json_rpc_result<T>(json: &str) -> T
where
    T: DeserializeOwned,
{
    load_json::<JsonRpcResult<T>>(json).result
}
