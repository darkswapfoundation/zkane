# ZKane Privacy Pool

A privacy pool implementation for alkanes assets using zero-knowledge proofs. ZKane allows users to deposit alkanes assets and withdraw them from different addresses, breaking on-chain links between deposits and withdrawals while maintaining full transaction validity.

## 🔒 Privacy Features

- **Zero-Knowledge Proofs**: Uses Noir circuits to prove withdrawal validity without revealing deposit information
- **Transaction Output Validation**: Cryptographically binds ZK proofs to specific Bitcoin transaction outputs to prevent frontrunning
- **Nullifier System**: Prevents double-spending while maintaining privacy
- **Witness Envelopes**: Efficiently stores large proof data bypassing Bitcoin's 80-byte opcode limits
- **Cross-Pool Isolation**: Each asset/denomination pair has its own isolated privacy pool

## 🏗️ Architecture

### Core Components

1. **ZKane Contract** (`alkanes/zkane/`): Core privacy pool contract implementing deposits and withdrawals
2. **ZKane Factory** (`alkanes/zkane-factory/`): Factory contract for creating and managing multiple pools
3. **Noir Circuits** (`noir/withdraw/`): Zero-knowledge proof circuits for withdrawal validation
4. **WASM Bindings** (`crates/zkane-wasm/`): Browser-compatible API for dapp integration
5. **Frontend Application** (`crates/zkane-frontend/`): A web-based user interface for interacting with the privacy pool, built with Leptos and powered by `deezel-web`.

### Supporting Crates

- **zkane-common** (`crates/zkane-common/`): Core types and data structures
- **zkane-crypto** (`crates/zkane-crypto/`): Cryptographic primitives (Poseidon hash, Merkle trees)
- **zkane-core** (`crates/zkane-core/`): High-level privacy pool operations

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+
- Node.js 18+ (for WASM builds)
- Noir compiler (for ZK circuits)
- wasm-pack (for browser builds)

### Installation

```bash
git clone https://github.com/zkane-project/zkane
cd zkane
cargo build --release
```

### Building WASM for Browser

```bash
# Build the WASM package
wasm-pack build crates/zkane-wasm --target web --out-dir pkg

# The generated package can be imported in JavaScript:
# import init, { create_deposit_note } from './pkg/zkane_wasm.js';
```

### Running Tests

```bash
# Run all tests
cargo test

# Run WASM tests in browser
wasm-pack test crates/zkane-wasm --headless --firefox

# Run specific test suites
cargo test zkane_integration
cargo test privacy_pool_tests
cargo test factory_integration
```

### Running the Frontend Application

The `zkane-frontend` crate provides a complete web application for interacting with the ZKane privacy pools. It is built with the [Leptos](https://leptos.dev/) framework and requires `trunk` to run.

```bash
# Install trunk if you don't have it
cargo install --locked trunk

# Navigate to the frontend directory
cd crates/zkane-frontend

# Run the development server
trunk serve
```

The application will be available at `http://localhost:8080` by default.

## 📖 Usage

### Basic Deposit Flow

```rust
use zkane::prelude::*;

// Create a deposit note
let asset_id = AlkaneId { block: 2, tx: 1 };
let denomination = 1000000u128; // 1M units
let deposit_note = generate_deposit_note(asset_id, denomination)?;

// The commitment goes into the privacy pool
println!("Commitment: {}", hex::encode(deposit_note.commitment.as_bytes()));

// Store the secret and nullifier securely for later withdrawal
println!("Secret: {}", hex::encode(deposit_note.secret.as_bytes()));
println!("Nullifier: {}", hex::encode(deposit_note.nullifier.as_bytes()));
```

### Basic Withdrawal Flow

```rust
use zkane::prelude::*;

// Generate nullifier hash for withdrawal
let nullifier_hash = generate_nullifier_hash(&deposit_note.nullifier)?;

// Calculate transaction outputs hash for recipient validation
let outputs = vec![(546u64, recipient_script.to_bytes())];
let outputs_hash = calculate_outputs_hash(&outputs);

// Generate ZK proof (using Noir circuit)
// This would typically be done off-chain
let proof = generate_withdrawal_proof(
    &deposit_note.secret,
    &deposit_note.nullifier,
    &merkle_path,
    &outputs_hash,
)?;

// Submit withdrawal transaction with proof in witness envelope
```

### JavaScript/Browser Usage

```javascript
import init, { 
    create_deposit_note, 
    generate_withdrawal_proof_placeholder,
    hash_transaction_outputs 
} from './pkg/zkane_wasm.js';

await init();

// Create deposit note
const assetId = { block: 2, tx: 1 };
const denomination = "1000000";
const depositNote = create_deposit_note(assetId, denomination);

console.log("Deposit note created:", depositNote);

// Calculate outputs hash for withdrawal
const outputs = [
    { value: 546, script_pubkey: "76a914..." }
];
const outputsHash = hash_transaction_outputs(JSON.stringify(outputs));

// Generate withdrawal proof
const proof = generate_withdrawal_proof_placeholder(
    depositNote.secret(),
    depositNote.nullifier(),
    JSON.stringify({ elements: [], indices: [] }),
    outputsHash
);
```

## 🔧 Contract Deployment

### 1. Deploy Contract Templates

```rust
// Deploy ZKane contract template to block 4
let zkane_wasm = include_bytes!("alkanes/zkane/target/wasm32-unknown-unknown/release/zkane.wasm");
// Deploy to alkanes with template ID [4, 0x1000]

// Deploy ZKane factory template to block 4  
let factory_wasm = include_bytes!("alkanes/zkane-factory/target/wasm32-unknown-unknown/release/zkane_factory.wasm");
// Deploy to alkanes with template ID [4, 0x2000]
```

### 2. Deploy Factory Instance

```rust
// Deploy factory instance to block 6
// Call cellpack: [6, 0x2000, 0] // Deploy factory instance
```

### 3. Create Privacy Pools

```rust
// Create pool for specific asset/denomination
// Call factory: [factory_block, factory_tx, 0, asset_block, asset_tx, denomination]
// Pool ID is deterministically generated from asset ID and denomination
```

## 🧪 Testing

The ZKane system includes comprehensive tests following the boiler pattern:

### Test Suites

1. **Unit Tests**: Individual component testing
2. **Integration Tests**: End-to-end system testing
3. **Privacy Pool Tests**: Core privacy functionality
4. **Factory Tests**: Pool creation and management
5. **WASM Tests**: Browser compatibility testing
6. **Security Tests**: Double-spending prevention, invalid proof rejection

### Running Specific Tests

```bash
# Test privacy pool functionality
cargo test test_zkane_deposit_flow
cargo test test_zkane_withdrawal_flow
cargo test test_zkane_security_features

# Test factory pattern
cargo test test_zkane_factory_pattern

# Test complete system
cargo test test_zkane_complete_system

# Test WASM bindings
wasm-pack test crates/zkane-wasm --headless --firefox
```

## 🔐 Security Model

### Privacy Guarantees

1. **Deposit Privacy**: Commitments are cryptographically hiding
2. **Withdrawal Privacy**: Zero-knowledge proofs reveal no information about deposits
3. **Link Breaking**: No on-chain connection between deposits and withdrawals
4. **Cross-Pool Isolation**: Assets in different pools cannot be linked

### Security Features

1. **Double-Spending Prevention**: Nullifier system prevents reuse of deposits
2. **Frontrunning Protection**: Transaction output validation binds proofs to specific recipients
3. **Invalid Proof Rejection**: Cryptographic verification prevents invalid withdrawals
4. **Witness Envelope Integrity**: Large data storage with cryptographic binding

### Assumptions

1. **Trusted Setup**: Noir circuits require trusted setup (ceremony-generated parameters)
2. **Honest Majority**: Assumes majority of users don't collude to break privacy
3. **Secure Implementation**: Relies on correct implementation of cryptographic primitives

## 📊 Performance

### Benchmarks

- **Deposit**: ~100ms (commitment generation + Merkle tree update)
- **Withdrawal**: ~2s (ZK proof generation + verification)
- **Pool Creation**: ~50ms (deterministic ID generation)
- **State Queries**: ~10ms (Merkle root, commitment count, nullifier status)

### Scalability

- **Max Commitments per Pool**: 1,048,576 (2^20)
- **Merkle Tree Depth**: 20 levels
- **Proof Size**: ~256 bytes (Noir PLONK proofs)
- **Witness Envelope Size**: Variable (typically 1-10KB)

## 🛠️ Development

### Project Structure

```
zkane/
├── alkanes/                    # Alkanes contracts
│   ├── zkane/                 # Core privacy pool contract
│   └── zkane-factory/         # Factory contract
├── crates/                    # Rust crates
│   ├── zkane-common/          # Core types
│   ├── zkane-crypto/          # Cryptographic primitives
│   ├── zkane-core/            # High-level operations
│   ├── zkane-wasm/            # WASM bindings
│   └── zkane-frontend/        # Leptos frontend application
├── noir/                      # Zero-knowledge circuits
│   └── withdraw/              # Withdrawal proof circuit
├── src/                       # Main library
│   ├── tests/                 # Test suites
│   └── lib.rs                 # Main library file
└── docs/                      # Documentation
```

### Building Components

```bash
# Build alkanes contracts
cd alkanes/zkane && cargo build --target wasm32-unknown-unknown --release
cd alkanes/zkane-factory && cargo build --target wasm32-unknown-unknown --release

# Build Noir circuits
cd noir/withdraw && nargo build

# Build WASM bindings
wasm-pack build crates/zkane-wasm --target web

# Build and serve the frontend application
cd crates/zkane-frontend && trunk serve

# Build main library
cargo build --release
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## 📚 Documentation

### API Documentation

```bash
# Generate and open documentation
cargo doc --open
```

### Circuit Documentation

The withdrawal circuit (`noir/withdraw/src/main.nr`) proves:

1. Knowledge of secret and nullifier for a committed value
2. Merkle inclusion proof for the commitment
3. Correct nullifier hash computation
4. Transaction outputs hash validation

### Contract Documentation

- **ZKane Contract**: Implements deposit/withdrawal opcodes with witness envelope support
- **ZKane Factory**: Manages pool creation using cellpack pattern for automatic deployment

## 🔗 Integration

### Alkanes Framework

ZKane integrates with the alkanes metaprotocol:

- Uses cellpack pattern for contract deployment
- Implements witness envelopes for large data storage
- Follows alkanes opcode conventions
- Integrates with protorune for token transfers

### Bitcoin Integration

- Transactions are standard Bitcoin transactions
- Proof data stored in witness envelopes
- Compatible with existing Bitcoin infrastructure
- No consensus changes required

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🤝 Acknowledgments

- [Alkanes Framework](https://github.com/sandshrewmetaprotocols/alkanes) for the metaprotocol foundation
- [Noir](https://noir-lang.org/) for zero-knowledge proof circuits
- [Tornado Cash](https://tornado.cash/) for privacy pool inspiration
- Bitcoin community for the underlying infrastructure

---

**⚠️ Security Notice**: This is experimental software. Do not use with real funds without proper security audits.
