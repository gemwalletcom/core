use crate::rpc::ankr::Transaction;

pub struct AnkrMapper {}

impl AnkrMapper {
    pub fn map_transactions_ids(transactions: Vec<Transaction>) -> Vec<String> {
        transactions.into_iter().map(|x| x.hash).collect()
    }
}
