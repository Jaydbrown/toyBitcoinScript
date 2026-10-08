# Jay's Bitcoin Node

A lightweight Bitcoin Script execution engine, JSON-RPC node interface, and peer-to-peer (P2P) gossip network written in Rust.

---

## Overview

Jay's Bitcoin Node is a modular implementation of core Bitcoin primitives built from scratch. It models the end-to-end lifecycle of Bitcoin data, from low-level opcode evaluation and raw transaction serialization to block mining and decentralized peer-to-peer message gossiping.

The system is split into four primary layers:
1. **Script Engine:** A stack-based virtual machine executing Bitcoin Script bytecode with standard consensus rules.
2. **Transaction Layer:** Binary encoding and decoding of transactions according to the Bitcoin wire protocol.
3. **JSON-RPC Server:** A standard JSON-RPC 2.0 interface exposing endpoints for script evaluation, transaction creation, mining, and node inspection.
4. **P2P Gossip Network:** An asynchronous TCP networking layer allowing multiple running nodes to discover each other, flood-relay transactions, and maintain synchronized mempools.

---

## Features

### 1. Bitcoin Script Engine
- Implements primary execution stack and secondary alt-stack (`OP_TOALTSTACK`, `OP_FROMALTSTACK`).
- Bytecode pushdata parsing (`0x00`, `0x01..=0x4b`, and opcodes).
- Stack manipulation opcodes: `OP_DUP`, `OP_DROP`, `OP_SWAP`, `OP_VERIFY`, `OP_EQUAL`, `OP_EQUALVERIFY`.
- Arithmetic operations: `OP_1ADD`, `OP_1SUB`, `OP_ADD`, `OP_SUB`.
- Cryptographic hashing: `OP_HASH160` (SHA-256 followed by RIPEMD-160).
- Bitcoin-compliant sign-magnitude numerical encoding and decoding (`encode_num`, `decode_num`).
- Bitcoin truthiness validation (`is_true`), including negative zero rejection.

### 2. Transaction Serialization
- Models `Tx`, `TxIn`, and `TxOut` with exact binary layouts.
- Little-endian integer encoding for versions, output values (satoshis), sequences, and locktimes.
- Wire parser with boundary checks to prevent buffer underflow or out-of-bounds panics.
- Unsigned transaction construction and hex serialization.

### 3. Mining & Cryptography
- Standard 80-byte `BlockHeader` layout (version, previous block hash, Merkle root, timestamp, difficulty bits, and nonce).
- Full Merkle tree reduction algorithm (`compute_merkle_root`), including duplication of the final hash when the count is odd.
- Proof of Work verification using double-SHA256 hashing.
- Built-in nonce mining loop (`mine` and `generate`) with configurable difficulty.

### 4. JSON-RPC 2.0 Server
- Built on `jsonrpsee` with asynchronous request routing.
- Hex decoding and encoding helper functions with odd-length and character validation.
- Method routing across four distinct modules: State, Raw Transactions, Node Info, and Mining.

### 5. P2P Gossip Networking
- Asynchronous TCP communication powered by Tokio.
- Wire protocol using newline-delimited JSON packets (`P2pMessage`).
- Deduplication cache (`seen_transactions`) to prevent infinite message relay loops.
- Live tracking of connected peers and dynamic mempool byte sizing.

---

## Project Structure

```
src/
├── lib.rs                  # Crate root re-exporting core modules
├── main.rs                 # Node entrypoint, CLI argument parser, and server runner
├── opcode.rs               # Opcode enum definitions and byte conversions
├── executionStack.rs       # Dual-stack data structures and helper methods
├── transaction.rs          # Tx, TxIn, and TxOut data models
├── rpc/
│   ├── mod.rs              # Re-exports all RPC traits and server implementations
│   ├── state.rs            # eval_script endpoint and hex encoding/decoding
│   ├── raw_transactions.rs # decoderawtransaction and createrawtransaction
│   ├── node.rs             # getblockchaininfo, getnetworkinfo, getmempoolinfo
│   └── mining/
│       ├── mod.rs          # Re-exports mining and Merkle tree utilities
│       ├── merkle.rs       # double_sha256 and compute_merkle_root logic
│       └── mining.rs       # BlockHeader, getblocktemplate, submitblock, generate
└── p2p/
    ├── mod.rs              # Re-exports Swarm and P2pMessage
    ├── message.rs          # P2pMessage wire packet enum
    ├── peer.rs             # TCP reader and writer tasks
    └── swarm.rs            # Peer pool, broadcast manager, and deduplication
```

---

## Installation & Requirements

Ensure you have Rust and Cargo installed (version 1.75 or later recommended).

Clone the repository and verify the build:
```bash
cargo check
cargo build
```

---

## Running the Node

### Single Node (Default)
To run a standalone node:
```bash
cargo run
```
By default, the node initializes:
- JSON-RPC server listening on `http://127.0.0.1:9944`
- P2P Swarm listener on `127.0.0.1:8001`

### Running a Multi-Node Swarm
You can run multiple instances on the same machine by passing CLI arguments:
```bash
cargo run -- [rpc_port] [p2p_port] [peer_address_to_connect]
```

**Terminal 1 (Bootstrap Node):**
```bash
cargo run -- 9944 8001
```

**Terminal 2 (Peer Node):**
```bash
cargo run -- 9945 8002 127.0.0.1:8001
```
The second node will bind its RPC to port 9945, listen for P2P on port 8002, and automatically connect to Node 1 on port 8001. Any transactions submitted to Node 1 will gossip to Node 2 automatically.

---

## JSON-RPC API Reference

All requests follow the standard JSON-RPC 2.0 specification via HTTP POST.

### 1. `eval_script`
Evaluates Bitcoin Script bytecode or an unlocking/locking script pair.

**Request:**
```bash
curl -X POST http://127.0.0.1:9944 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eval_script",
    "params": {
      "script_hex": "525393"
    },
    "id": 1
  }'
```
*Note: `525393` corresponds to `OP_2 OP_3 OP_ADD`.*

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "success": true,
    "final_stack": ["05"],
    "error": null
  },
  "id": 1
}
```

---

### 2. `createrawtransaction`
Constructs an unsigned raw transaction hex string from given inputs and outputs.

**Request:**
```bash
curl -X POST http://127.0.0.1:9944 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "createrawtransaction",
    "params": {
      "inputs": [
        {
          "tx_id_hex": "0000000000000000000000000000000000000000000000000000000000000001",
          "vout": 0,
          "sequence": 4294967295
        }
      ],
      "outputs": [
        {
          "value": 50000000,
          "script_hex": "76a914000000000000000000000000000000000000000088ac"
        }
      ],
      "locktime": 0
    },
    "id": 1
  }'
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": "010000000100000000000000000000000000000000000000000000000000000000000000010000000000ffffffff0100f20502000000001976a914000000000000000000000000000000000000000088ac00000000",
  "id": 1
}
```

---

### 3. `decoderawtransaction`
Parses raw transaction hex into readable JSON.

**Request:**
```bash
curl -X POST http://127.0.0.1:9944 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "decoderawtransaction",
    "params": ["<raw_hex_string>"],
    "id": 1
  }'
```

---

### 4. `getblockchaininfo`
Returns current chain state, block height, best hash, and verification progress.

**Request:**
```bash
curl -X POST http://127.0.0.1:9944 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "getblockchaininfo", "id": 1}'
```

---

### 5. `getnetworkinfo`
Returns node version, subversion, protocol version, and live connected peer count.

**Request:**
```bash
curl -X POST http://127.0.0.1:9944 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "getnetworkinfo", "id": 1}'
```

---

### 6. `getmempoolinfo`
Returns dynamic transaction count, cumulative byte size, and memory usage.

**Request:**
```bash
curl -X POST http://127.0.0.1:9944 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "getmempoolinfo", "id": 1}'
```

---

### 7. `getblocktemplate`
Returns the previous block hash, difficulty target bits, and transaction list for miners.

**Request:**
```bash
curl -X POST http://127.0.0.1:9944 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "getblocktemplate", "id": 1}'
```

---

### 8. `generate`
Instructs the node to execute the mining loop, find a winning nonce, and output the mined block.

**Request:**
```bash
curl -X POST http://127.0.0.1:9944 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "generate",
    "params": [1],
    "id": 1
  }'
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "nonce": 341,
    "block_hash": "00c7e2b19f...",
    "header_hex": "010000000000...",
    "height": 1
  },
  "id": 1
}
```

---

### 9. `submitblock`
Submits a block header and transactions. Verifies Merkle root consistency and difficulty before accepting.

**Request:**
```bash
curl -X POST http://127.0.0.1:9944 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "submitblock",
    "params": {
      "header_hex": "<80_byte_header_hex>",
      "tx_hexes": []
    },
    "id": 1
  }'
```

---

## Testing

Run unit and integration tests across the codebase:
```bash
cargo test
```

---

## Contributing

Feel free to open a PR if you feel the need to add any features or modify. Direct pushes to `master` are protected—all changes must be submitted via a Pull Request.

### Contribution Workflow:
1. Fork or branch from `master`:
   ```bash
   git checkout -b feature/your-feature-name
   ```
2. Commit and push your changes to your branch:
   ```bash
   git push origin feature/your-feature-name
   ```
3. Open a Pull Request on GitHub against `master`.

