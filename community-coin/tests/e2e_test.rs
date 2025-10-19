use community_coin::blockchain::CommunityBlockchain;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use reqwest;

#[tokio::test]
async fn test_api_endpoints() {
    // 1. Setup
    let mut initial_wallets = HashMap::new();
    initial_wallets.insert("alice".to_string(), 1000);
    initial_wallets.insert("bob".to_string(), 500);
    let db_path = "test_db_api_endpoints";
    // Clean up any previous test runs
    let _ = std::fs::remove_dir_all(db_path);
    let blockchain = CommunityBlockchain::new(initial_wallets, db_path).unwrap();
    let blockchain = Arc::new(RwLock::new(blockchain));

    // 2. Start server in a background task
    tokio::spawn(async move {
        community_coin::start_server(blockchain, 8001).await.unwrap();
    });

    // 3. Test /wallet/:address endpoint
    let alice_wallet: serde_json::Value = reqwest::get("http://localhost:8001/wallet/alice")
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(alice_wallet["balance"], 1000);

    // 4. Test /transfer endpoint
    let transfer_body = serde_json::json!({
        "from": "alice",
        "to": "bob",
        "amount": 100
    });
    let transfer_response = reqwest::Client::new()
        .post("http://localhost:8001/transfer")
        .json(&transfer_body)
        .send()
        .await
        .unwrap();
    assert!(transfer_response.status().is_success());

    // 5. Test /mine endpoint
    let mine_body = serde_json::json!({
        "proposer": "miner"
    });
    let mine_response = reqwest::Client::new()
        .post("http://localhost:8001/mine")
        .json(&mine_body)
        .send()
        .await
        .unwrap();
    assert!(mine_response.status().is_success());

    // Extract the new block from the response
    let mine_response_json: serde_json::Value = mine_response.json().await.unwrap();
    let new_block = mine_response_json.get("block").unwrap();

    // 5.5. Test /add-block endpoint
    let add_block_response = reqwest::Client::new()
        .post("http://localhost:8001/add-block")
        .json(new_block)
        .send()
        .await
        .unwrap();
    assert!(add_block_response.status().is_success());

    // 6. Verify balances after mining
    let bob_wallet: serde_json::Value = reqwest::get("http://localhost:8001/wallet/bob")
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(bob_wallet["balance"], 600);

    // 7. Clean up the database
    let _ = std::fs::remove_dir_all(db_path);
}
