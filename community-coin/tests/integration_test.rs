use community_coin::blockchain::CommunityBlockchain;
use std::collections::HashMap;

#[test]
fn test_full_transaction_and_mining_flow() {
    // 1. Setup
    let mut initial_wallets = HashMap::new();
    initial_wallets.insert("alice".to_string(), 1000);
    initial_wallets.insert("bob".to_string(), 500);
    let db_path = "test_db_full_flow";
    let blockchain = CommunityBlockchain::new(initial_wallets, db_path).unwrap();

    // 2. Alice sends 100 coins to Bob
    let tx_id = blockchain
        .create_transaction("alice".to_string(), "bob".to_string(), 100)
        .unwrap();
    assert!(!tx_id.is_empty());

    // 3. Check pending transactions
    let pending_txs = blockchain.get_pending();
    assert_eq!(pending_txs.len(), 1);
    assert_eq!(pending_txs[0].tx_id, tx_id);

    // 4. Mine a block
    let block = blockchain.mine_block("miner".to_string()).unwrap();
    blockchain.add_block(block).unwrap();

    // 5. Verify balances
    let alice_balance = blockchain.get_balance("alice").unwrap();
    let bob_balance = blockchain.get_balance("bob").unwrap();
    assert_eq!(alice_balance, 899); // 1000 - 100 (amount) - 1 (fee)
    assert_eq!(bob_balance, 600); // 500 + 100 (amount)

    // 6. Check pending transactions are cleared
    let pending_txs_after_mining = blockchain.get_pending();
    assert!(pending_txs_after_mining.is_empty());

    // 7. Clean up the database
    let _ = std::fs::remove_dir_all(db_path);
}
