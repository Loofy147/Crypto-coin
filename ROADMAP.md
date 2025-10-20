# Community Coin Development Roadmap

This document outlines the planned development phases for the Community Coin blockchain. Our goal is to build a secure, decentralized, and feature-rich platform.

---

### Phase 1: Foundational Infrastructure (Complete)

This phase focused on establishing the core, single-node functionality of the blockchain.

- **[✓] Local Ledger:** Implemented a persistent ledger using Sled DB.
- **[✓] Wallet Management:** Created a wallet system with deterministic key generation from wallet addresses.
- **[✓] Transaction Signing:** Implemented robust Ed25519 signature creation and verification using `ed25519-dalek`.
- **[✓] Basic REST API:** Developed a web server using Axum to expose core functionalities like creating transactions, checking balances, and viewing the chain.

---

### Phase 2: Decentralization & Consensus

This phase will transform the blockchain from a single-node application into a true distributed network.

- **[ ] P2P Networking Layer:**
  - Implement a peer-to-peer networking layer for nodes to discover each other and exchange information.
  - *Technology*: `libp2p` is a strong candidate for its modularity and robustness.
- **[ ] Consensus Algorithm:**
  - Design and implement a consensus mechanism to ensure all nodes agree on the state of the ledger.
  - *Initial Plan*: Start with a simple Proof-of-Authority (PoA) model with a defined set of validator nodes, then explore options for Proof-of-Stake (PoS).
- **[ ] Block & Transaction Propagation:**
  - Develop the logic for broadcasting new blocks and transactions across the network.
  - Implement a network-wide mempool to manage pending transactions.
- **[ ] State Synchronization:**
  - Create a mechanism for new nodes to join the network and securely download the entire history of the blockchain.

---

### Phase 3: Smart Contracts & Advanced Features

This phase will introduce programmable logic to the blockchain, enabling the development of decentralized applications (dApps).

- **[ ] Virtual Machine (VM) Integration:**
  - Integrate a WebAssembly (WASM) runtime to execute smart contracts in a sandboxed environment.
  - *Technology*: `Wasmer` or `Wasmtime` are leading candidates.
- **[ ] Smart Contract Framework:**
  - Define the API for smart contracts to interact with the blockchain state (e.g., read/write to storage, access block information).
  - Implement logic for deploying and executing contracts.
- **[ ] State Trie Implementation:**
  - Replace the current simple state management with a Merkle Patricia Trie (or similar structure) for efficient and verifiable state lookups.
- **[ ] Gas Model:**
  - Introduce a gas mechanism to charge fees for transaction execution and smart contract operations, preventing infinite loops and rewarding validators.

---

### Phase 4: Ecosystem & Tooling

This phase focuses on building the tools and resources necessary for developers and users to interact with the Community Coin network.

- **[ ] Advanced CLI:**
  - Develop a command-line interface for users to manage wallets, send transactions, deploy contracts, and query network state.
- **[ ] JavaScript/TypeScript SDK:**
  - Create a software development kit to enable easy integration of Community Coin with web and Node.js applications.
- **[ ] Block Explorer:**
  - Build a web-based tool for visualizing blocks, transactions, and smart contract state on the blockchain.
- **[ ] Documentation Portal:**
  - Launch a comprehensive documentation website with tutorials, API references, and architectural overviews.
