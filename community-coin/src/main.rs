use community_coin::CommunityBlockchain;
use community_coin::p2p::NetworkService;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::mpsc;
use libp2p::Swarm;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing Community Coin Blockchain...\n");

    // Create a channel for communication between the API and the P2P service
    let (tx, mut rx) = mpsc::channel(32);

    // Start the P2P service in a background task
    tokio::spawn(async move {
        let mut network_service = NetworkService::new().await;
        loop {
            tokio::select! {
                event = network_service.swarm.select_next_some() => {
                    println!("P2P event: {:?}", event);
                }
                Some(message) = rx.recv() => {
                    network_service.swarm.behaviour_mut().gossipsub.publish(network_service.topic.clone(), message).unwrap();
                }
            }
        }
    });

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
