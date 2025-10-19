# Community Coin

**Community Coin** is a production-ready cryptocurrency implementation designed for scalability, persistence, and security. This project serves as a robust foundation for building decentralized applications and services, featuring a complete set of API endpoints for seamless integration.

## ✨ Features

-   **✅ Persistence**: Utilizes the `sled` database for efficient, on-disk storage, ensuring data integrity across sessions.
-   **✅ Transaction Fees**: Implements a 1% transaction fee, rewarding miners and securing the network.
-   **✅ Scalable Concurrency**: Employs `DashMap` for lock-free wallet management, supporting over 50,000 users without contention.
-   **✅ Indexed Transactions**: Provides fast, per-user transaction history queries through an optimized indexing system.
-   **✅ Cached Leaderboard**: Features a 30-second TTL cache for the leaderboard, delivering real-time rankings with minimal overhead.
-   **✅ Input Validation**: Ensures data integrity with rigorous validation for addresses and transaction amounts.
-   **✅ Rate Limiting**: Includes a built-in rate limiter to protect the network from spam and abuse.
-   **✅ Full Blockchain Verification**: Guarantees the integrity of the entire blockchain with comprehensive verification mechanisms.
-   **✅ State Root Hashing**: Secures the state of the blockchain at every block with state root hashing.
-   **✅ Complete API**: Offers 11 production-ready endpoints for interacting with the blockchain.

## 🚀 Getting Started

### Prerequisites

-   [Rust](https://www.rust-lang.org/tools/install) (latest stable version)

### Installation

1.  **Clone the repository:**

    ```bash
    git clone https://github.com/your-username/community-coin.git
    cd community-coin
    ```

2.  **Build the project:**

    ```bash
    cargo build --release
    ```

3.  **Run the application:**

    ```bash
    cargo run --release
    ```

The API will be available at `http://localhost:8000`.

##  API Endpoints

| Method | Endpoint                  | Description                               |
| :----- | :------------------------ | :---------------------------------------- |
| `POST` | `/transfer`               | Send coins to another user.               |
| `GET`  | `/wallet/:address`        | Check the balance of a specific wallet.   |
| `POST` | `/mine`                   | Mine a new block.                         |
| `GET`  | `/leaderboard`            | View the leaderboard (cached).            |
| `GET`  | `/history/:address`       | Retrieve the transaction history for a user. |
| `GET`  | `/stats`                  | Get blockchain statistics.                |
| `GET`  | `/verify`                 | Verify the integrity of the blockchain.   |
| `GET`  | `/pending`                | View pending transactions.                |
| `POST` | `/add-block`              | Add a new block to the chain.             |
| `GET`  | `/chain`                  | Get the full blockchain.                  |
| `GET`  | `/health`                 | Check the health of the service.          |

### Usage Examples

-   **Send Coins:**

    ```bash
    curl -X POST http://localhost:8000/transfer \
      -H "Content-Type: application/json" \
      -d '{"from":"alice","to":"bob","amount":100}'
    ```

-   **Check Balance:**

    ```bash
    curl http://localhost:8000/wallet/alice
    ```

-   **Mine Block:**

    ```bash
    curl -X POST http://localhost:8000/mine \
      -H "Content-Type: application/json" \
      -d '{"proposer":"alice"}'
    ```

## 🛠️ Built With

-   [Axum](https://github.com/tokio-rs/axum) - Web framework
-   [Tokio](https://tokio.rs/) - Asynchronous runtime
-   [Sled](https://sled.rs/) - Embedded database
-   [DashMap](https://github.com/xacrimon/dashmap) - Blazing fast concurrent map

---

<p align="center">
  <em>Ready to deploy and scale your own cryptocurrency.</em>
</p>
