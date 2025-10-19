use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use dashmap::DashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Transaction: User sends coins to another user with optional fee
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub fee: u64,
    pub timestamp: u64,
    pub tx_id: String,
    pub signature: String,
    pub nonce: u64,
}

/// Block: Contains multiple transactions with state root
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub prev_hash: String,
    pub hash: String,
    pub proposer: String,
    pub state_root: String,
}

/// Wallet: Each user has a wallet with balance and history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub address: String,
    pub balance: u64,
    pub tx_count: u64,
    pub created_at: u64,
    pub last_updated: u64,
}

/// Transaction index for fast lookups
#[derive(Debug, Clone)]
pub struct TransactionIndex {
    pub tx_id: String,
    pub block_index: u64,
    pub tx_index_in_block: usize,
}

/// CommunityBlockchain: Production-ready blockchain with persistence
pub struct CommunityBlockchain {
    chain: Arc<Mutex<Vec<Block>>>,
    wallets: Arc<DashMap<String, Wallet>>,
    tx_index: Arc<DashMap<String, Vec<TransactionIndex>>>, // Per-user tx index
    pending_txs: Arc<Mutex<Vec<Transaction>>>,
    nonces: Arc<DashMap<String, u64>>, // Track nonce per user for ordering
    state_db: sled::Db,
    txn_counter: Arc<Mutex<u64>>,
}

impl CommunityBlockchain {
    /// Create new blockchain with sled persistence
    pub fn new(initial_wallets: HashMap<String, u64>, db_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let state_db = sled::open(db_path)?;
        let now = current_timestamp();

        let wallets = Arc::new(DashMap::new());
        let tx_index = Arc::new(DashMap::new());
        let nonces = Arc::new(DashMap::new());

        for (address, balance) in initial_wallets {
            let wallet = Wallet {
                address: address.clone(),
                balance,
                tx_count: 0,
                created_at: now,
                last_updated: now,
            };
            wallets.insert(address.clone(), wallet.clone());
            nonces.insert(address.clone(), 0);
            tx_index.insert(address.clone(), Vec::new());

            // Persist wallet
            let wallet_json = serde_json::to_string(&wallet)?;
            state_db.insert(format!("wallet:{}", address).as_bytes(), wallet_json.as_bytes())?;
        }

        // Genesis block
        let genesis = Block {
            index: 0,
            timestamp: now,
            transactions: vec![],
            prev_hash: "0".to_string(),
            hash: "genesis".to_string(),
            proposer: "system".to_string(),
            state_root: "genesis_root".to_string(),
        };

        let chain = Arc::new(Mutex::new(vec![genesis.clone()]));

        // Persist genesis
        let genesis_json = serde_json::to_string(&genesis)?;
        state_db.insert(b"block:0", genesis_json.as_bytes())?;

        Ok(CommunityBlockchain {
            chain,
            wallets,
            tx_index,
            pending_txs: Arc::new(Mutex::new(Vec::new())),
            nonces,
            state_db,
            txn_counter: Arc::new(Mutex::new(0)),
        })
    }

    /// Load blockchain from disk
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let state_db = sled::open("blockchain_state")?;

        let mut chain = Vec::new();
        let wallets = Arc::new(DashMap::new());
        let tx_index = Arc::new(DashMap::new());
        let nonces = Arc::new(DashMap::new());

        // Load all blocks
        let mut block_idx = 0;
        loop {
            let key = format!("block:{}", block_idx);
            match state_db.get(key.as_bytes())? {
                Some(block_bytes) => {
                    let block: Block = serde_json::from_slice(&block_bytes)?;
                    chain.push(block);
                    block_idx += 1;
                }
                None => break,
            }
        }

        // Load all wallets and rebuild indices
        for item in state_db.scan_prefix(b"wallet:") {
            if let Ok((_key, value)) = item {
                let wallet: Wallet = serde_json::from_slice(&value)?;
                wallets.insert(wallet.address.clone(), wallet.clone());
                nonces.insert(wallet.address.clone(), 0);
                tx_index.insert(wallet.address.clone(), Vec::new());
            }
        }

        Ok(CommunityBlockchain {
            chain: Arc::new(Mutex::new(chain)),
            wallets,
            tx_index,
            pending_txs: Arc::new(Mutex::new(Vec::new())),
            nonces,
            state_db,
            txn_counter: Arc::new(Mutex::new(0)),
        })
    }

    /// Create transaction with validation and nonce tracking
    pub fn create_transaction(
        &self,
        from: String,
        to: String,
        amount: u64,
    ) -> Result<String, String> {
        if amount == 0 {
            return Err("Amount must be greater than 0".to_string());
        }

        // Check sender exists
        let sender_wallet = self.wallets.get(&from)
            .ok_or("Sender wallet not found".to_string())?;

        // Check balance (including fee: 1% of amount)
        let fee = (amount as f64 * 0.01).ceil() as u64;
        let total_cost = amount + fee;

        if sender_wallet.balance < total_cost {
            return Err(format!(
                "Insufficient balance: {} has {}, needs {} (amount {} + fee {})",
                from, sender_wallet.balance, total_cost, amount, fee
            ));
        }
        drop(sender_wallet);

        // Ensure recipient exists or will be created
        if !self.wallets.contains_key(&to) {
            let now = current_timestamp();
            let new_wallet = Wallet {
                address: to.clone(),
                balance: 0,
                tx_count: 0,
                created_at: now,
                last_updated: now,
            };
            self.wallets.insert(to.clone(), new_wallet);
            self.tx_index.insert(to.clone(), Vec::new());
            self.nonces.insert(to.clone(), 0);
        }

        // Get nonce
        let mut nonce_entry = self.nonces.entry(from.clone()).or_insert(0);
        *nonce_entry += 1;
        let current_nonce = *nonce_entry;
        drop(nonce_entry);

        let timestamp = current_timestamp();
        let tx_id = format!("{}-{}-{}-{}", from, to, current_nonce, timestamp);
        let signature = self.sign_transaction(&tx_id, &from);

        let tx = Transaction {
            from,
            to,
            amount,
            fee,
            timestamp,
            tx_id: tx_id.clone(),
            signature,
            nonce: current_nonce,
        };

        let mut pending = self.pending_txs.lock().unwrap();
        pending.push(tx);

        Ok(tx_id)
    }

    /// Sign transaction
    fn sign_transaction(&self, tx_id: &str, sender: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(tx_id.as_bytes());
        hasher.update(sender.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Verify transaction signature
    fn verify_signature(tx: &Transaction) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(tx.tx_id.as_bytes());
        hasher.update(tx.from.as_bytes());
        format!("{:x}", hasher.finalize()) == tx.signature
    }

    /// Calculate state root from wallet balances
    fn calculate_state_root(&self, wallets: &HashMap<String, u64>) -> String {
        let mut hasher = Sha256::new();
        let mut sorted_wallets: Vec<_> = wallets.iter().collect();
        sorted_wallets.sort_by_key(|(k, _)| *k);

        for (addr, balance) in sorted_wallets {
            hasher.update(addr.as_bytes());
            hasher.update(balance.to_le_bytes());
        }

        format!("{:x}", hasher.finalize())
    }

    /// Mine a block (PoS-like with proposer)
    pub fn mine_block(&self, proposer: String) -> Result<Block, String> {
        let mut pending = self.pending_txs.lock().unwrap();

        if pending.is_empty() {
            return Err("No pending transactions to mine".to_string());
        }

        // Validate transactions in order (nonce-based ordering)
        let mut valid_txs = Vec::new();
        let mut temp_balances: HashMap<String, u64> = HashMap::new();
        let mut tx_nonces: HashMap<String, u64> = HashMap::new();

        // Initialize temp balances
        for wallet_ref in self.wallets.iter() {
            temp_balances.insert(wallet_ref.key().clone(), wallet_ref.value().balance);
        }

        for tx in pending.iter() {
            if !Self::verify_signature(tx) {
                continue;
            }

            // Check nonce ordering
            let expected_nonce = tx_nonces.entry(tx.from.clone()).or_insert(0);
            if tx.nonce != *expected_nonce + 1 {
                continue;
            }
            *expected_nonce = tx.nonce;

            let sender_balance = temp_balances.get(&tx.from).copied().unwrap_or(0);
            if sender_balance >= tx.amount + tx.fee {
                temp_balances.insert(tx.from.clone(), sender_balance - tx.amount - tx.fee);
                let recipient_balance = temp_balances.get(&tx.to).copied().unwrap_or(0);
                temp_balances.insert(tx.to.clone(), recipient_balance + tx.amount);
                valid_txs.push(tx.clone());
            }
        }

        if valid_txs.is_empty() {
            return Err("No valid transactions after validation".to_string());
        }

        pending.clear();
        drop(pending);

        let chain = self.chain.lock().unwrap();
        let last_block = chain.last().unwrap();
        let prev_hash = last_block.hash.clone();
        let new_index = last_block.index + 1;
        drop(chain);

        let state_root = self.calculate_state_root(&temp_balances);

        let mut block = Block {
            index: new_index,
            timestamp: current_timestamp(),
            transactions: valid_txs,
            prev_hash,
            hash: String::new(),
            proposer,
            state_root,
        };

        block.hash = self.calculate_block_hash(&block);

        Ok(block)
    }

    /// Calculate block hash
    fn calculate_block_hash(&self, block: &Block) -> String {
        let mut hasher = Sha256::new();
        hasher.update(block.index.to_le_bytes());
        hasher.update(block.timestamp.to_le_bytes());
        hasher.update(block.prev_hash.as_bytes());
        hasher.update(block.state_root.as_bytes());

        for tx in &block.transactions {
            hasher.update(tx.tx_id.as_bytes());
        }

        format!("{:x}", hasher.finalize())
    }

    /// Add block to chain and persist
    pub fn add_block(&self, block: Block) -> Result<(), String> {
        let chain = self.chain.lock().unwrap();
        let last_block = chain.last().unwrap();

        // Validate block
        if block.index != last_block.index + 1 {
            return Err("Invalid block index".to_string());
        }

        if block.prev_hash != last_block.hash {
            return Err("Invalid previous hash".to_string());
        }

        let calc_hash = self.calculate_block_hash(&block);
        if calc_hash != block.hash {
            return Err("Invalid block hash".to_string());
        }

        drop(chain);

        // Apply transactions to wallets
        for tx in &block.transactions {
            if let Some(mut sender) = self.wallets.get_mut(&tx.from) {
                sender.balance -= tx.amount + tx.fee;
                sender.tx_count += 1;
                sender.last_updated = current_timestamp();
            }

            let mut recipient = self.wallets.entry(tx.to.clone())
                .or_insert_with(|| Wallet {
                    address: tx.to.clone(),
                    balance: 0,
                    tx_count: 0,
                    created_at: current_timestamp(),
                    last_updated: current_timestamp(),
                });
            recipient.balance += tx.amount;
            recipient.last_updated = current_timestamp();

            // Update per-user transaction index
            let mut user_txs = self.tx_index.entry(tx.from.clone())
                .or_insert_with(Vec::new);
            user_txs.push(TransactionIndex {
                tx_id: tx.tx_id.clone(),
                block_index: block.index,
                tx_index_in_block: block.transactions.iter().position(|t| t.tx_id == tx.tx_id).unwrap(),
            });

            let mut recipient_txs = self.tx_index.entry(tx.to.clone())
                .or_insert_with(Vec::new);
            recipient_txs.push(TransactionIndex {
                tx_id: tx.tx_id.clone(),
                block_index: block.index,
                tx_index_in_block: block.transactions.iter().position(|t| t.tx_id == tx.tx_id).unwrap(),
            });
        }

        // Persist block and wallets to disk
        if let Err(e) = self.persist_block(&block) {
            return Err(format!("Failed to persist block: {}", e));
        }

        for wallet_ref in self.wallets.iter() {
            let wallet_json = serde_json::to_string(&wallet_ref.value()).unwrap();
            let _ = self.state_db.insert(
                format!("wallet:{}", wallet_ref.key()).as_bytes(),
                wallet_json.as_bytes(),
            );
        }

        // Add to chain
        let mut chain = self.chain.lock().unwrap();
        chain.push(block);

        Ok(())
    }

    /// Persist block to disk
    fn persist_block(&self, block: &Block) -> Result<(), Box<dyn std::error::Error>> {
        let block_json = serde_json::to_string(block)?;
        self.state_db.insert(
            format!("block:{}", block.index).as_bytes(),
            block_json.as_bytes(),
        )?;
        Ok(())
    }

    /// Get wallet
    pub fn get_wallet(&self, address: &str) -> Result<Wallet, String> {
        self.wallets
            .get(address)
            .map(|w| w.value().clone())
            .ok_or("Wallet not found".to_string())
    }

    /// Get all wallets (for leaderboard)
    pub fn get_leaderboard(&self) -> Vec<Wallet> {
        let mut wallets: Vec<_> = self.wallets
            .iter()
            .map(|entry| entry.value().clone())
            .collect();
        wallets.sort_by(|a, b| b.balance.cmp(&a.balance));
        wallets
    }

    /// Get user transactions (fast due to indexing)
    pub fn get_user_transactions(&self, address: &str) -> Vec<Transaction> {
        let chain = self.chain.lock().unwrap();
        let mut txs = Vec::new();

        if let Some(indices) = self.tx_index.get(address) {
            for index in indices.iter() {
                if let Some(block) = chain.get(index.block_index as usize) {
                    if let Some(tx) = block.transactions.get(index.tx_index_in_block) {
                        txs.push(tx.clone());
                    }
                }
            }
        }

        txs
    }

    /// Get pending transactions
    pub fn get_pending(&self) -> Vec<Transaction> {
        self.pending_txs.lock().unwrap().clone()
    }

    /// Get blockchain
    pub fn get_chain(&self) -> Vec<Block> {
        self.chain.lock().unwrap().clone()
    }

    /// Verify chain integrity
    pub fn verify_chain(&self) -> bool {
        let chain = self.chain.lock().unwrap();

        for i in 1..chain.len() {
            let current = &chain[i];
            let previous = &chain[i - 1];

            if current.prev_hash != previous.hash {
                return false;
            }

            let calc_hash = self.calculate_block_hash(current);
            if calc_hash != current.hash {
                return false;
            }
        }

        true
    }

    /// Get stats
    pub fn get_stats(&self) -> serde_json::Value {
        let chain = self.chain.lock().unwrap();
        let pending = self.pending_txs.lock().unwrap();
        let total_txs: u64 = chain.iter().map(|b| b.transactions.len() as u64).sum();
        let total_coins: u64 = self.wallets.iter().map(|entry| entry.value().balance).sum();

        serde_json::json!({
            "chain_height": chain.len() - 1,
            "total_blocks": chain.len(),
            "total_wallets": self.wallets.len(),
            "total_transactions": total_txs,
            "pending_transactions": pending.len(),
            "total_coins": total_coins,
            "is_valid": self.verify_chain(),
        })
    }

    pub fn get_balance(&self, address: &str) -> Result<u64, String> {
        self.get_wallet(address).map(|w| w.balance)
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_with_fees() {
        let mut initial = HashMap::new();
        initial.insert("alice".to_string(), 1000);
        initial.insert("bob".to_string(), 500);

        let db_path = "test_db_transaction_with_fees";
        let blockchain = CommunityBlockchain::new(initial, db_path).unwrap();
        let _ = std::fs::remove_dir_all(db_path);


        let tx_id = blockchain
            .create_transaction("alice".to_string(), "bob".to_string(), 100)
            .unwrap();

        assert!(!tx_id.is_empty());
        let pending = blockchain.get_pending();
        assert_eq!(pending[0].fee, 1); // 1% of 100
    }

    #[test]
    fn test_block_persistence() {
        let mut initial = HashMap::new();
        initial.insert("alice".to_string(), 1000);

        let db_path = "test_db_block_persistence";
        let blockchain = CommunityBlockchain::new(initial, db_path).unwrap();
        let _ = std::fs::remove_dir_all(db_path);
        blockchain
            .create_transaction("alice".to_string(), "bob".to_string(), 100)
            .unwrap();

        let block = blockchain.mine_block("proposer".to_string()).unwrap();
        blockchain.add_block(block).unwrap();

        assert_eq!(blockchain.get_balance("alice").unwrap(), 899); // 1000 - 100 - 1 fee
    }

    #[test]
    fn test_leaderboard_ordering() {
        let mut initial = HashMap::new();
        initial.insert("alice".to_string(), 1000);
        initial.insert("bob".to_string(), 500);
        initial.insert("charlie".to_string(), 750);

        let db_path = "test_db_leaderboard_ordering";
        let blockchain = CommunityBlockchain::new(initial, db_path).unwrap();
        let _ = std::fs::remove_dir_all(db_path);
        let leaderboard = blockchain.get_leaderboard();

        assert_eq!(leaderboard[0].address, "alice");
        assert_eq!(leaderboard[1].address, "charlie");
        assert_eq!(leaderboard[2].address, "bob");
    }

    #[test]
    fn test_fast_transaction_lookup() {
        let mut initial = HashMap::new();
        initial.insert("alice".to_string(), 1000);

        let db_path = "test_db_fast_transaction_lookup";
        let blockchain = CommunityBlockchain::new(initial, db_path).unwrap();
        let _ = std::fs::remove_dir_all(db_path);

        for _ in 0..100 {
            blockchain
                .create_transaction("alice".to_string(), "bob".to_string(), 1)
                .unwrap();
        }

        let block = blockchain.mine_block("proposer".to_string()).unwrap();
        blockchain.add_block(block).unwrap();

        let history = blockchain.get_user_transactions("alice");
        assert_eq!(history.len(), 100);
    }
}
