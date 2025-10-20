use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use dashmap::DashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use ed25519_dalek::{Signature, SigningKey, VerifyingKey};
use signature::{Signer, Verifier};
use std::fmt;

// --- Transaction, Block, Wallet Structs ---

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

#[derive(Serialize, Deserialize)]
pub struct Wallet {
    pub address: String,
    pub public_key: [u8; 32],
    #[serde(skip)]
    pub keypair: Option<SigningKey>,
    pub balance: u64,
    pub tx_count: u64,
    pub created_at: u64,
    pub last_updated: u64,
}

// --- Wallet Implementations ---
impl Wallet {
    pub fn new(address: String, balance: u64) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(address.as_bytes());
        let seed: [u8; 32] = hasher.finalize().into();
        let signing_key = SigningKey::from_bytes(&seed);
        let public_key = signing_key.verifying_key();
        let now = current_timestamp();
        Wallet {
            address,
            public_key: public_key.to_bytes(),
            keypair: Some(signing_key),
            balance,
            tx_count: 0,
            created_at: now,
            last_updated: now,
        }
    }
}

impl Clone for Wallet {
    fn clone(&self) -> Self {
        let keypair = if self.keypair.is_some() {
            let mut hasher = Sha256::new();
            hasher.update(self.address.as_bytes());
            let seed: [u8; 32] = hasher.finalize().into();
            let signing_key = SigningKey::from_bytes(&seed);
            Some(signing_key)
        } else {
            None
        };

        Wallet {
            address: self.address.clone(),
            public_key: self.public_key,
            keypair,
            balance: self.balance,
            tx_count: self.tx_count,
            created_at: self.created_at,
            last_updated: self.last_updated,
        }
    }
}

impl fmt::Debug for Wallet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Wallet")
            .field("address", &self.address)
            //.field("public_key", &self.public_key) // Omitted for brevity
            .field("balance", &self.balance)
            .field("tx_count", &self.tx_count)
            .finish()
    }
}


// --- Other Structs and Blockchain Implementation ---

#[derive(Debug, Clone)]
pub struct TransactionIndex {
    pub tx_id: String,
    pub block_index: u64,
    pub tx_index_in_block: usize,
}

pub struct CommunityBlockchain {
    chain: Arc<Mutex<Vec<Block>>>,
    wallets: Arc<DashMap<String, Wallet>>,
    tx_index: Arc<DashMap<String, Vec<TransactionIndex>>>,
    pending_txs: Arc<Mutex<Vec<Transaction>>>,
    nonces: Arc<DashMap<String, u64>>,
    state_db: sled::Db,
}

impl CommunityBlockchain {
    pub fn new(initial_wallets: HashMap<String, u64>, db_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let state_db = sled::open(db_path)?;
        let wallets = Arc::new(DashMap::new());
        for (address, balance) in initial_wallets {
            wallets.insert(address.clone(), Wallet::new(address, balance));
        }

        let genesis = Block {
            index: 0,
            timestamp: current_timestamp(),
            transactions: vec![],
            prev_hash: "0".to_string(),
            hash: "genesis".to_string(),
            proposer: "system".to_string(),
            state_root: "genesis_root".to_string(),
        };
let chain = Arc::new(Mutex::new(vec![genesis]));

        Ok(CommunityBlockchain {
            chain,
            wallets,
            tx_index: Arc::new(DashMap::new()),
            pending_txs: Arc::new(Mutex::new(Vec::new())),
            nonces: Arc::new(DashMap::new()),
            state_db,
        })
    }

    pub fn load(db_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let state_db = sled::open(db_path)?;
        let mut chain: Vec<Block> = Vec::new();
        for item in state_db.scan_prefix(b"block:") {
            if let Ok((_key, value)) = item {
                let block: Block = serde_json::from_slice(&value)?;
                chain.push(block);
            }
        }
        chain.sort_by_key(|b| b.index);

        let wallets = Arc::new(DashMap::new());
        for item in state_db.scan_prefix(b"wallet:") {
            if let Ok((_key, value)) = item {
                let mut wallet: Wallet = serde_json::from_slice(&value)?;
                let mut hasher = Sha256::new();
                hasher.update(wallet.address.as_bytes());
                let seed: [u8; 32] = hasher.finalize().into();
                let signing_key = SigningKey::from_bytes(&seed);
                wallet.keypair = Some(signing_key);
                wallets.insert(wallet.address.clone(), wallet);
            }
        }

        Ok(CommunityBlockchain {
            chain: Arc::new(Mutex::new(chain)),
            wallets,
            tx_index: Arc::new(DashMap::new()),
            pending_txs: Arc::new(Mutex::new(Vec::new())),
            nonces: Arc::new(DashMap::new()),
            state_db,
        })
    }

    pub fn create_transaction(&self, from: String, to: String, amount: u64) -> Result<String, String> {
        let sender_wallet = self.wallets.get(&from).ok_or("Sender not found")?;
        let fee = 1; // Simplified fee
        if sender_wallet.balance < amount + fee {
            return Err("Insufficient balance".to_string());
        }

        let timestamp = current_timestamp();
        let tx_id = format!("{}-{}-{}", from, to, timestamp);

        let signature = {
            let keypair = sender_wallet.keypair.as_ref().unwrap();
            let signature: Signature = keypair.sign(tx_id.as_bytes());
            base64::encode(signature.to_bytes())
        };

        let tx = Transaction {
            from, to, amount, fee, timestamp, tx_id: tx_id.clone(), signature, nonce: 0,
        };
        self.pending_txs.lock().unwrap().push(tx);
        Ok(tx_id)
    }

    fn verify_signature(&self, tx: &Transaction) -> bool {
        if let Some(sender_wallet) = self.wallets.get(&tx.from) {
            if let Ok(signature_bytes) = base64::decode(&tx.signature) {
                if let Ok(signature_array) = <&[u8; 64]>::try_from(signature_bytes.as_slice()) {
                    let signature = Signature::from_bytes(signature_array);
                    if let Ok(public_key) = VerifyingKey::from_bytes(&sender_wallet.public_key) {
                        return public_key.verify(tx.tx_id.as_bytes(), &signature).is_ok();
                    }
                }
            }
        }
        false
    }

    // ... (Include stubs for all other blockchain functions)
    pub fn mine_block(&self, proposer: String) -> Result<Block, String> {
        let mut pending_txs = self.pending_txs.lock().unwrap();
        let transactions_to_mine = pending_txs.clone();
        pending_txs.clear();

        let (index, prev_hash) = {
            let chain = self.chain.lock().unwrap();
            let last_block = chain.last().unwrap();
            (last_block.index + 1, last_block.hash.clone())
        };

        let timestamp = current_timestamp();
        let hash = format!("{}-{}-{}", index, timestamp, prev_hash); // Simplified hash

        let block = Block {
            index,
            timestamp,
            transactions: transactions_to_mine,
            prev_hash,
            hash,
            proposer,
            state_root: "not_implemented".to_string(),
        };

        Ok(block)
    }

    pub fn add_block(&self, block: Block) -> Result<(), String> {
        // In a real implementation, we would do more validation here.
        let mut chain = self.chain.lock().unwrap();
        chain.push(block.clone());

        // Persist the block and update wallets
        self.state_db.insert(
            format!("block:{}", block.index).as_bytes(),
            serde_json::to_vec(&block).unwrap(),
        ).unwrap();

        for tx in &block.transactions {
            if let Some(mut sender) = self.wallets.get_mut(&tx.from) {
                sender.balance -= tx.amount + tx.fee;
            }
            if let Some(mut receiver) = self.wallets.get_mut(&tx.to) {
                receiver.balance += tx.amount;
            } else {
                // If the receiver doesn't exist, create a new wallet for them.
                let new_wallet = Wallet::new(tx.to.clone(), tx.amount);
                self.wallets.insert(tx.to.clone(), new_wallet);
            }
        }

        // Persist wallet changes
        for wallet_entry in self.wallets.iter() {
            let (address, wallet) = wallet_entry.pair();
            self.state_db.insert(
                format!("wallet:{}", address).as_bytes(),
                serde_json::to_vec(wallet).unwrap(),
            ).unwrap();
        }


        Ok(())
    }
    pub fn get_balance(&self, address: &str) -> Result<u64, String> {
        self.wallets
            .get(address)
            .map(|w| w.balance)
            .ok_or_else(|| "Wallet not found".to_string())
    }
    pub fn verify_chain(&self) -> bool {
        let chain = self.chain.lock().unwrap();
        for i in 1..chain.len() {
            if chain[i].prev_hash != chain[i-1].hash {
                return false;
            }
        }
        true
    }
    pub fn get_wallet(&self, address: &str) -> Result<Wallet, String> {
        self.wallets
            .get(address)
            .map(|w| w.clone())
            .ok_or_else(|| "Wallet not found".to_string())
    }
    pub fn get_leaderboard(&self) -> Vec<Wallet> {
        let mut wallets: Vec<Wallet> = self.wallets.iter().map(|w| w.value().clone()).collect();
        wallets.sort_by(|a, b| b.balance.cmp(&a.balance));
        wallets
    }
    pub fn get_user_transactions(&self, address: &str) -> Vec<Transaction> {
        let chain = self.chain.lock().unwrap();
        chain
            .iter()
            .flat_map(|b| b.transactions.clone())
            .filter(|tx| tx.from == address || tx.to == address)
            .collect()
    }
    pub fn get_pending(&self) -> Vec<Transaction> {
        self.pending_txs.lock().unwrap().clone()
    }
    pub fn get_chain(&self) -> Vec<Block> {
        self.chain.lock().unwrap().clone()
    }
    pub fn get_stats(&self) -> serde_json::Value {
        let chain = self.chain.lock().unwrap();
        let num_txs: usize = chain.iter().map(|b| b.transactions.len()).sum();
        serde_json::json!({
            "blocks": chain.len(),
            "transactions": num_txs,
            "wallets": self.wallets.len(),
        })
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static DB_COUNTER: AtomicUsize = AtomicUsize::new(0);


    fn get_unique_db_path() -> String {
        let count = DB_COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = format!("/tmp/test_db_{}", count);
        if std::path::Path::new(&path).exists() {
            std::fs::remove_dir_all(&path).unwrap();
        }
        path
    }

    #[test]
    fn test_signature_verification_after_reload() {
        let db_path = get_unique_db_path();
        let mut initial = HashMap::new();
        initial.insert("alice".to_string(), 1000);

        // 1. Create a blockchain and a transaction
        {
            let blockchain = CommunityBlockchain::new(initial, &db_path).unwrap();
            blockchain
                .create_transaction("alice".to_string(), "bob".to_string(), 100)
                .unwrap();

            // 2. Mine a block and add it to the chain
            let block = blockchain.mine_block("miner1".to_string()).unwrap();
            blockchain.add_block(block).unwrap();
        } // blockchain goes out of scope here, and its data should be persisted.

        // 3. Load the blockchain from the database
        let blockchain = CommunityBlockchain::load(&db_path).unwrap();

        // 4. Create another transaction from the re-loaded wallet
        let tx_id = blockchain
            .create_transaction("alice".to_string(), "charlie".to_string(), 50)
            .unwrap();

        // 5. Verify the signature of the new transaction
        let pending_txs = blockchain.pending_txs.lock().unwrap();
        let tx = pending_txs.iter().find(|t| t.tx_id == tx_id).unwrap();
        assert!(blockchain.verify_signature(tx));
    }
}
