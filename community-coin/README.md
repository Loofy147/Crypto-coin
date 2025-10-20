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

## 💻 CLI Wallet

Community Coin also includes a command-line interface (CLI) wallet for easy interaction with the blockchain.

### Installation

The CLI is built along with the main application:

```bash
cargo build --release
```

### Usage

-   **Check Balance:**

    ```bash
    ./target/release/cli wallet balance <ADDRESS>
    ```

-   **Transfer Coins:**

    ```bash
    ./target/release/cli wallet transfer --from <FROM> --to <TO> --amount <AMOUNT> --private-key <PRIVATE_KEY>
    ```

-   **Get History:**

    ```bash
    ./target/release/cli wallet history <ADDRESS>
    ```

## 📝 Smart Contracts

Community Coin supports general-purpose smart contracts written in any language that can be compiled to WebAssembly (Wasm).

### Writing Smart Contracts

Smart contracts interact with the blockchain through a defined ABI. An example "counter" contract written in Rust can be found in the `example-contract` directory.

### Deploying Smart Contracts

Use the CLI to deploy a smart contract:

```bash
./target/release/cli contract deploy --path <PATH_TO_WASM_FILE> --private-key <PRIVATE_KEY>
```

### Executing Smart Contracts

Use the CLI to execute a function on a deployed smart contract:

```bash
./target/release/cli contract execute --contract-id <CONTRACT_ID> --function <FUNCTION_NAME> --args <ARGS_JSON> --private-key <PRIVATE_KEY>
```

## 🛡️ Shared Security

Community Coin uses a shared security model inspired by EigenLayer to secure the network. Validators stake assets on a simulated Settlement Layer, and their attestations are required to validate new blocks.

### Becoming a Validator

To become a validator, you must stake assets on the Settlement Layer. This can be done via the CLI:

```bash
./target/release/cli validator stake --amount <AMOUNT> --private-key <PRIVATE_KEY>
```

### Slashing

Validators who act maliciously (e.g., by signing conflicting blocks) will have their stake slashed.

## 🌐 P2P Networking

Community Coin uses `libp2p` to create a peer-to-peer network for discovering other nodes and sharing transactions and blocks.

### Running Multiple Nodes

To run multiple nodes on the same machine, you can specify a different API port and P2P port for each node:

**Node 1:**

```bash
cargo run --release -- --api-port 8000 --p2p-port 10000
```

**Node 2:**

```bash
cargo run --release -- --api-port 8001 --p2p-port 10001
```

The nodes will automatically discover each other on the local network using mDNS.

## 🛠️ Built With

-   [Axum](https://github.com/tokio-rs/axum) - Web framework
-   [Tokio](https://tokio.rs/) - Asynchronous runtime
-   [Sled](https://sled.rs/) - Embedded database
-   [DashMap](https://github.com/xacrimon/dashmap) - Blazing fast concurrent map

---

<p align="center">
  <em>Ready to deploy and scale your own cryptocurrency.</em>
</p>
