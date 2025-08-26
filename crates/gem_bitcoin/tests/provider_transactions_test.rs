use chain_traits::{ChainState, ChainTransactions};
use primitives::{TransactionState, TransactionStateRequest};

use crate::testkit::TEST_TRANSACTION_ID;

mod testkit;

#[tokio::test]
async fn test_bitcoin_get_transactions_status() {
    let bitcoin_client = testkit::create_bitcoin_test_client();

    let request = TransactionStateRequest::new_id(TEST_TRANSACTION_ID.to_string());
    let update = bitcoin_client.get_transaction_status(request).await.unwrap();

    println!("State: {}", update.state);
    assert!(update.state == TransactionState::Confirmed);
}

#[tokio::test]
async fn test_bitcoin_get_transactions_by_block() {
    let bitcoin_client = testkit::create_bitcoin_test_client();

    let latest_block = bitcoin_client.get_block_latest_number().await.unwrap();
    let transactions = bitcoin_client.get_transactions_by_block(latest_block).await.unwrap();

    println!("Latest block: {}, transactions count: {}", latest_block, transactions.len());
    assert!(latest_block > 0);
}

#[tokio::test]
async fn test_bitcoin_get_transactions_by_address() {
    let bitcoin_client = testkit::create_bitcoin_test_client();

    let transactions = bitcoin_client.get_transactions_by_address(testkit::TEST_ADDRESS.to_string()).await.unwrap();

    println!("Address: {}, transactions count: {}", testkit::TEST_ADDRESS, transactions.len());
}
