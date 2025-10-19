use community_coin::CommunityBlockchain;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing Community Coin Blockchain...\n");

    // Load or create blockchain
    let blockchain = match CommunityBlockchain::load() {
        Ok(bc) => {
            println!("✓ Loaded existing blockchain from disk");
            bc
        }
        Err(_) => {
            println!("✓ Creating new blockchain");
            let mut initial = std::collections::HashMap::new();

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

    let blockchain = Arc::new(RwLock::new(blockchain));

    // Start server on port 8000
    community_coin::start_server(blockchain, 8000).await?;

    Ok(())
}
