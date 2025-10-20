pub mod blockchain;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use dashmap::DashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use self::blockchain::CommunityBlockchain;

/// Rate limiter
#[derive(Clone)]
pub struct RateLimiter {
    buckets: Arc<DashMap<String, (f64, u64)>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        RateLimiter {
            buckets: Arc::new(DashMap::new()),
        }
    }

    pub fn check(&self, key: String, limit: f64, window_secs: u64) -> bool {
        let now = current_timestamp();
        let mut entry = self.buckets.entry(key).or_insert_with(|| (limit, now));

        let time_passed = now - entry.1;
        entry.0 += (time_passed as f64 / window_secs as f64) * limit;
        entry.0 = entry.0.min(limit);
        entry.1 = now;

        if entry.0 >= 1.0 {
            entry.0 -= 1.0;
            true
        } else {
            false
        }
    }
}

/// Leaderboard cache
#[derive(Clone)]
pub struct LeaderboardCache {
    cache: Arc<RwLock<Option<Vec<blockchain::Wallet>>>>,
    last_update: Arc<RwLock<u64>>,
    ttl_secs: u64,
}

impl LeaderboardCache {
    pub fn new(ttl_secs: u64) -> Self {
        LeaderboardCache {
            cache: Arc::new(RwLock::new(None)),
            last_update: Arc::new(RwLock::new(0)),
            ttl_secs,
        }
    }

    pub async fn get_or_update(&self, wallets: Vec<blockchain::Wallet>) -> Vec<blockchain::Wallet> {
        let now = current_timestamp();
        let last_update = *self.last_update.read().await;

        if now - last_update < self.ttl_secs {
            if let Some(cached) = self.cache.read().await.as_ref() {
                return cached.clone();
            }
        }

        *self.cache.write().await = Some(wallets.clone());
        *self.last_update.write().await = now;
        wallets
    }

    pub async fn invalidate(&self) {
        *self.cache.write().await = None;
    }
}

#[derive(Clone)]
pub struct AppState {
    blockchain: Arc<RwLock<CommunityBlockchain>>,
    leaderboard_cache: LeaderboardCache,
}

#[derive(Serialize, Deserialize)]
pub struct TransferRequest {
    pub from: String,
    pub to: String,
    pub amount: u64,
}

#[derive(Serialize, Deserialize)]
pub struct MineBlockRequest {
    pub proposer: String,
}

/// Validators
fn validate_address(addr: &str) -> Result<(), String> {
    if addr.is_empty() || addr.len() > 255 {
        return Err("Invalid address".to_string());
    }
    if !addr.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err("Address contains invalid characters".to_string());
    }
    Ok(())
}

fn validate_amount(amount: u64) -> Result<(), String> {
    if amount == 0 {
        return Err("Amount must be > 0".to_string());
    }
    if amount > 1_000_000_000_000 {
        return Err("Amount exceeds limit".to_string());
    }
    Ok(())
}

/// Transfer endpoint
pub async fn transfer(
    State(state): State<AppState>,
    Json(req): Json<TransferRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    if let Err(e) = validate_address(&req.from) {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": e})));
    }
    if let Err(e) = validate_address(&req.to) {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": e})));
    }
    if let Err(e) = validate_amount(req.amount) {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": e})));
    }

    let blockchain = state.blockchain.write().await;
    match blockchain.create_transaction(req.from, req.to, req.amount) {
        Ok(tx_id) => {
            state.leaderboard_cache.invalidate().await;
            (StatusCode::OK, Json(json!({"success": true, "tx_id": tx_id, "status": "pending"})))
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"success": false, "error": e}))),
    }
}

/// Get wallet
pub async fn get_wallet(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    if let Err(e) = validate_address(&address) {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": e})));
    }

    let blockchain = state.blockchain.read().await;
    match blockchain.get_wallet(&address) {
        Ok(wallet) => (
            StatusCode::OK,
            Json(json!({
                "address": wallet.address,
                "balance": wallet.balance,
                "tx_count": wallet.tx_count,
                "created_at": wallet.created_at,
            })),
        ),
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({"error": "Wallet not found"}))),
    }
}

/// Get leaderboard (cached)
pub async fn leaderboard(
    State(state): State<AppState>,
) -> (StatusCode, Json<Vec<serde_json::Value>>) {
    let blockchain = state.blockchain.read().await;
    let wallets = blockchain.get_leaderboard();
    let cached = state.leaderboard_cache.get_or_update(wallets).await;

    let result: Vec<_> = cached
        .iter()
        .enumerate()
        .map(|(rank, w)| {
            json!({
                "rank": rank + 1,
                "address": w.address,
                "balance": w.balance,
                "tx_count": w.tx_count,
            })
        })
        .collect();

    (StatusCode::OK, Json(result))
}

/// Get transaction history (uses index for speed)
pub async fn history(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> (StatusCode, Json<Vec<serde_json::Value>>) {
    if let Err(e) = validate_address(&address) {
        return (StatusCode::BAD_REQUEST, Json(vec![json!({"error": e})]));
    }

    let blockchain = state.blockchain.read().await;
    let txs = blockchain.get_user_transactions(&address);

    let result: Vec<_> = txs
        .iter()
        .map(|tx| {
            json!({
                "tx_id": tx.tx_id,
                "from": tx.from,
                "to": tx.to,
                "amount": tx.amount,
                "fee": tx.fee,
                "timestamp": tx.timestamp,
                "type": if tx.from == address { "sent" } else { "received" },
            })
        })
        .collect();

    (StatusCode::OK, Json(result))
}

/// Get pending transactions
pub async fn pending(
    State(state): State<AppState>,
) -> (StatusCode, Json<Vec<serde_json::Value>>) {
    let blockchain = state.blockchain.read().await;
    let pending_txs = blockchain.get_pending();

    let result: Vec<_> = pending_txs
        .iter()
        .map(|tx| {
            json!({
                "tx_id": tx.tx_id,
                "from": tx.from,
                "to": tx.to,
                "amount": tx.amount,
                "fee": tx.fee,
                "timestamp": tx.timestamp,
            })
        })
        .collect();

    (StatusCode::OK, Json(result))
}

/// Mine block
pub async fn mine_block(
    State(state): State<AppState>,
    Json(req): Json<MineBlockRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    if let Err(e) = validate_address(&req.proposer) {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": e})));
    }

    let blockchain = state.blockchain.write().await;
    match blockchain.mine_block(req.proposer) {
        Ok(block) => {
            state.leaderboard_cache.invalidate().await;
            (
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "block": {
                        "index": block.index,
                        "hash": block.hash,
                        "prev_hash": block.prev_hash,
                        "timestamp": block.timestamp,
                        "transactions": block.transactions.len(),
                        "state_root": block.state_root,
                    }
                })),
            )
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"success": false, "error": e}))),
    }
}

/// Add block to chain
pub async fn add_block(
    State(state): State<AppState>,
    Json(block_json): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let block: blockchain::Block = match serde_json::from_value(block_json) {
        Ok(b) => b,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("Invalid block: {}", e)}))),
    };

    let blockchain = state.blockchain.write().await;
    match blockchain.add_block(block) {
        Ok(_) => {
            state.leaderboard_cache.invalidate().await;
            (StatusCode::OK, Json(json!({"success": true, "message": "Block added"})))
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"success": false, "error": e}))),
    }
}

/// Get full blockchain
pub async fn get_chain(
    State(state): State<AppState>,
) -> (StatusCode, Json<Vec<serde_json::Value>>) {
    let blockchain = state.blockchain.read().await;
    let chain = blockchain.get_chain();

    let result: Vec<_> = chain
        .iter()
        .map(|block| {
            json!({
                "index": block.index,
                "hash": block.hash,
                "prev_hash": block.prev_hash,
                "timestamp": block.timestamp,
                "transactions": block.transactions.len(),
                "state_root": block.state_root,
            })
        })
        .collect();

    (StatusCode::OK, Json(result))
}

/// Verify chain integrity
pub async fn verify(
    State(state): State<AppState>,
) -> (StatusCode, Json<serde_json::Value>) {
    let blockchain = state.blockchain.read().await;
    let is_valid = blockchain.verify_chain();

    (
        StatusCode::OK,
        Json(json!({
            "valid": is_valid,
            "message": if is_valid { "Blockchain is valid" } else { "Chain corrupted" }
        })),
    )
}

/// Get stats
pub async fn stats(
    State(state): State<AppState>,
) -> (StatusCode, Json<serde_json::Value>) {
    let blockchain = state.blockchain.read().await;
    let stats = blockchain.get_stats();

    (StatusCode::OK, Json(stats))
}

/// Health check
pub async fn health() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "status": "healthy",
            "service": "Community Coin Blockchain"
        })),
    )
}

/// Start server
pub async fn start_server(
    blockchain: Arc<RwLock<CommunityBlockchain>>,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState {
        blockchain,
        leaderboard_cache: LeaderboardCache::new(30), // 30 second TTL
    };

    let app = Router::new()
        .route("/wallet/:address", get(get_wallet))
        .route("/leaderboard", get(leaderboard))
        .route("/history/:address", get(history))
        .route("/transfer", post(transfer))
        .route("/pending", get(pending))
        .route("/mine", post(mine_block))
        .route("/add-block", post(add_block))
        .route("/chain", get(get_chain))
        .route("/verify", get(verify))
        .route("/stats", get(stats))
        .route("/health", get(health))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    println!("🚀 Community Coin Blockchain API running on http://0.0.0.0:{}", port);
    println!("\n📋 Endpoints:");
    println!("  GET    /wallet/{{address}}      - Get wallet balance");
    println!("  GET    /leaderboard             - Top wallets (cached 30s)");
    println!("  GET    /history/{{address}}      - Transaction history (indexed)");
    println!("  POST   /transfer                - Send coins");
    println!("  GET    /pending                 - Pending transactions");
    println!("  POST   /mine                    - Mine new block");
    println!("  POST   /add-block               - Add mined block");
    println!("  GET    /chain                   - Full blockchain");
    println!("  GET    /verify                  - Verify integrity");
    println!("  GET    /stats                   - Blockchain stats");
    println!("  GET    /health                  - Health check\n");

    axum::serve(listener, app).await?;
    Ok(())
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(not(test))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing Community Coin Blockchain...\n");

    // Load or create blockchain
    let blockchain = match CommunityBlockchain::load("blockchain_state") {
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
    start_server(blockchain, 8000).await?;

    Ok(())
}
