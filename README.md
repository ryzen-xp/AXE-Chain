# ⛓️ AXE-Chain

[![Build](https://github.com/ryzen-xp/Blockchain/actions/workflows/rust.yml/badge.svg)](https://github.com/ryzen-xp/Blockchain/actions/workflows/rust.yml)
[![Language](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A **Proof of Work blockchain** built from scratch in Rust. AXE-Chain implements the core primitives of a real blockchain — block mining, SHA-256 hashing, transaction management, a wallet, and a peer node — all organized as a clean Cargo workspace.

---

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Project Structure](#project-structure)
- [How It Works](#how-it-works)
- [Getting Started](#getting-started)
- [Usage](#usage)
- [Dependencies](#dependencies)
- [CI](#ci)
- [Contributing](#contributing)

---

## Overview

AXE-Chain is a learning-focused, ground-up implementation of a Proof of Work blockchain in Rust. It covers:

- Block creation with SHA-256 hashing
- Proof of Work mining with adjustable difficulty
- Transaction construction and signing
- A wallet for key management
- A node for chain state management

The goal is to demonstrate how real blockchains work under the hood — no shortcuts, no wrappers around existing chain libraries.

---

## Architecture

The project is a Cargo workspace with four crates, each with a single responsibility:

```
AXE-Chain/
├── lib/        # Core blockchain types and logic (blocks, chain, hashing, PoW)
├── miner/      # Mining loop — finds valid nonces, produces new blocks
├── node/       # Node state — holds the chain, validates incoming blocks
└── wallet/     # Key generation, address derivation, transaction signing
```

Each binary crate (`miner`, `node`, `wallet`) depends on `lib` for shared types. This keeps the consensus logic in one place and avoids duplication.

---

## Project Structure

```
AXE-Chain/
├── .github/
│   └── workflows/
│       └── rust.yml        # CI pipeline (build + test)
├── docs/                   # Design notes and diagrams
├── lib/                    # Core library crate
│   └── src/
│       ├── block.rs        # Block struct, hash computation
│       ├── blockchain.rs   # Chain management, validation
│       ├── transaction.rs  # Transaction types
│       └── pow.rs          # Proof of Work logic
├── miner/                  # Miner binary
│   └── src/main.rs
├── node/                   # Node binary
│   └── src/main.rs
├── wallet/                 # Wallet binary
│   └── src/main.rs
├── Cargo.toml              # Workspace manifest
└── Cargo.lock
```

---

## How It Works

### Blocks

Each block contains:

| Field | Description |
|---|---|
| `index` | Block height in the chain |
| `timestamp` | Unix timestamp of block creation |
| `transactions` | List of transactions included in the block |
| `previous_hash` | SHA-256 hash of the preceding block |
| `nonce` | The value that satisfies the PoW condition |
| `hash` | SHA-256 hash of this block's contents |

### Proof of Work

The miner repeatedly increments a `nonce` and hashes the block header until the resulting hash starts with a required number of leading zeroes — the **difficulty target**. This makes block production computationally expensive and forgery prohibitively hard.

```
hash(block_data + nonce) → 0000...XYZ  ✓
```

### Chain Validation

Every new block is validated against two rules before acceptance:

1. Its `previous_hash` must match the actual hash of the last block.
2. Its hash must satisfy the current difficulty target.

### Genesis Block

The first block (index `0`) is hardcoded with a fixed `previous_hash` of `"0"`. It is the only block without a real predecessor.

---

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable, 1.70+)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Clone

```bash
git clone https://github.com/ryzen-xp/AXE-Chain.git
cd AXE-Chain
```

### Build

```bash
# Build all workspace members
cargo build --release
```

### Test

```bash
cargo test
```

---

## Usage

### Run the miner

```bash
cargo run --release -p miner
```

The miner will begin finding valid nonces for new blocks and output each mined block to stdout.

### Run a node

```bash
cargo run --release -p node
```

The node maintains the canonical chain state and validates incoming blocks.

### Use the wallet

```bash
cargo run --release -p wallet
```

Generate a new keypair, derive an address, and sign transactions.

---

## Dependencies

Defined at the workspace level in `Cargo.toml` and shared across crates:

| Crate | Version | Purpose |
|---|---|---|
| `sha2` | 0.10 | SHA-256 block hashing |
| `chrono` | 0.4 | Block timestamps |
| `serde` | 1.0 | JSON serialization / deserialization |

---

## Contributing

Contributions are welcome. To get started:

1. Fork the repository
2. Create a feature branch (`git checkout -b feat/your-feature`)
3. Make your changes and add tests
4. Open a pull request with a clear description

Please keep code formatted with `rustfmt` and linted with `clippy` before submitting.

---

## Author

Built by [@ryzen-xp](https://github.com/ryzen-xp)
