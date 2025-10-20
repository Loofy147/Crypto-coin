//! A full node for the Community Coin sovereign rollup.

use community_coin::blockchain::CommunityBlockchain;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, RwLock};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Initializing Community Coin Full Node...\n");

    // Load or create blockchain
    let blockchain = match CommunityBlockchain::load("blockchain_state") {
        Ok(bc) => {
            println!("✓ Loaded existing blockchain from disk");
            bc
        }
        Err(_) => {
            println!("✓ Creating new blockchain");
            let mut initial = HashMap::new();

            // Initialize 50 demo users
            for i in 1..=50 {
                initial.insert(format!("user_{}", i), 1000u64);
            }

            // Add named accounts
            initial.insert("alice".to_string(), 10000);
            initial.insert("bob".to_string(), 5000);
            initial.insert("charlie".to_string(), 3000);

            CommunityBlockchain::new(initial, "blockchain_state")?
        }
    };

    let _blockchain = Arc::new(RwLock::new(blockchain));

    // The API server is defined in `main.rs`.
    // We need to figure out how to share the server logic.
    // For now, we'll just print a message.
    println!("Full node running. API server not yet implemented in this binary.");

    Ok(())
}
