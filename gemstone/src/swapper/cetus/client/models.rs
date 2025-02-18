use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct Request {
    pub display_all_pools: bool,
    pub has_mining: bool,
    pub no_incentives: bool,
    pub coin_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub code: u16,
    pub msg: String,
    pub data: ResponseData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseData {
    pub total: u32,
    pub lp_list: Vec<Pool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pool {
    pub address: String,
    pub pool_type: String,
    pub coin_a_address: String,
    pub coin_b_address: String,
    pub fee: String,
    pub name: String,
    pub tick_spacing: String,
    pub object: Object,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Object {
    pub current_sqrt_price: String,
    pub tick_spacing: i32,
    pub liquidity: String,
    pub is_pause: bool,
    pub index: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request() {
        let data = include_str!("test/stats_pool.json");
        let response: Response = serde_json::from_slice(data.as_bytes()).unwrap();

        assert_eq!(response.data.lp_list.len(), 1);
    }
}
