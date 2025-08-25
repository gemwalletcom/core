use chain_traits::{ChainState, ChainTransactions};

mod testkit;

#[tokio::test]
async fn test_get_transactions_by_block() {
    let polkadot_client = testkit::create_polkadot_test_client();

    let latest_block = polkadot_client.get_block_latest_number().await.unwrap();
    let transactions = polkadot_client.get_transactions_by_block(latest_block).await.unwrap();

    println!("Latest block: {}, transactions count: {}", latest_block, transactions.len());
    assert!(latest_block > 0);
}

#[tokio::test]
async fn test_get_transactions_by_address() {
    let polkadot_client = testkit::create_polkadot_test_client();

    let transactions = polkadot_client.get_transactions_by_address(testkit::TEST_ADDRESS.to_string()).await.unwrap();

    println!("Address: {}, transactions count: {}", testkit::TEST_ADDRESS, transactions.len());
}
