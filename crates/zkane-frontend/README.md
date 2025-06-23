# ZKane Frontend

A privacy-preserving web application for alkanes assets using zero-knowledge proofs, built with Leptos and WebAssembly.

## Overview

ZKane Frontend is a full-stack Rust web application that provides a user-friendly interface for interacting with ZKane privacy pools. It enables users to:

- **Deposit alkanes assets** into privacy pools with configurable denominations
- **Generate withdrawal proofs** using zero-knowledge cryptography
- **Manage deposit notes** securely in local storage
- **Browse privacy pools** and their anonymity sets
- **Track transaction history** and manage user preferences

## Architecture

### Technology Stack

- **Frontend Framework**: [Leptos](https://leptos.dev/) - Reactive web framework for Rust
- **WebAssembly**: Compiled Rust code running in the browser
- **Styling**: Custom CSS with CSS variables for theming
- **Build Tool**: [wasm-pack](https://rustwasm.github.io/wasm-pack/) for WASM compilation
- **Development Server**: basic-http-server for local development

### Project Structure

```
crates/zkane-frontend/
├── src/
│   ├── lib.rs              # WASM entry point and exports
│   ├── app.rs              # Main application component with routing
│   ├── types.rs            # Type definitions for frontend/backend integration
│   ├── services.rs         # Service layer integrating ZKane and alkanes
│   ├── components/         # UI components
│   │   ├── mod.rs          # Component module exports
│   │   ├── deposit.rs      # Deposit-related components
│   │   ├── withdraw.rs     # Withdrawal-related components
│   │   ├── utils.rs        # Utility components
│   │   └── ...             # Other component modules
│   └── styles.css          # Application styles
├── index.html              # Main HTML template
├── build.rs                # Build script for WASM compilation
├── Cargo.toml              # Dependencies and configuration
└── README.md               # This file
```

## Features

### 🔐 Privacy-First Design

- **Zero-Knowledge Proofs**: Generate withdrawal proofs without revealing deposit history
- **Local Storage**: Deposit notes stored securely in browser local storage
- **No Server Dependencies**: Fully client-side application with WASM

### 💰 Asset Management

- **Multi-Asset Support**: Works with any alkanes-compatible asset
- **Flexible Denominations**: Choose from available pool denominations
- **Balance Validation**: Real-time balance checking and validation

### 🎨 User Experience

- **Responsive Design**: Works on desktop and mobile devices
- **Dark/Light Themes**: Automatic theme detection with manual override
- **Real-time Updates**: Reactive UI updates based on blockchain state
- **Comprehensive Help**: Built-in documentation and security tips

### 🧪 Testing

- **Component Tests**: Individual component testing with Leptos
- **Integration Tests**: End-to-end workflow testing
- **WASM Tests**: Browser-based testing with wasm-bindgen-test
- **Performance Tests**: Component creation and rendering benchmarks

## Getting Started

### Prerequisites

1. **Rust**: Install from [rustup.rs](https://rustup.rs/)
2. **wasm-pack**: Install with `curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh`
3. **HTTP Server**: Install with `cargo install basic-http-server`

### Development Setup

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd zkane/crates/zkane-frontend
   ```

2. **Build the application**:
   ```bash
   # Using the build script (recommended)
   ../../scripts/build-frontend.sh debug
   
   # Or manually
   wasm-pack build --target web --dev
   ```

3. **Serve the application**:
   ```bash
   # The build script can serve automatically
   ../../scripts/build-frontend.sh debug
   
   # Or manually
   cd dist && basic-http-server
   ```

4. **Open in browser**: Navigate to `http://localhost:8080`

### Production Build

```bash
# Build optimized version
../../scripts/build-frontend.sh release

# Serve production build
cd dist && basic-http-server
```

## Usage Guide

### Making a Deposit

1. **Connect Wallet**: Ensure your alkanes wallet is connected
2. **Select Asset**: Choose an asset from your available balances
3. **Enter Amount**: Specify the amount to deposit
4. **Create Note**: Generate a deposit note (save this securely!)
5. **Submit Transaction**: Send the deposit transaction to the network

### Making a Withdrawal

1. **Load Deposit Note**: Paste or upload your saved deposit note
2. **Enter Recipient**: Specify the Bitcoin address to receive funds
3. **Generate Proof**: Create a zero-knowledge withdrawal proof
4. **Submit Transaction**: Send the withdrawal transaction to the network

### Security Best Practices

- **Save Deposit Notes**: Always save deposit notes securely
- **Use Different Networks**: Use different connections for deposits vs withdrawals
- **Wait for Anonymity**: Let more users join your pool before withdrawing
- **Verify Addresses**: Double-check recipient addresses before submitting

## API Reference

### Components

#### Core Components

- `App`: Main application with routing and layout
- `DepositComponent`: Complete deposit workflow
- `WithdrawComponent`: Complete withdrawal workflow
- `PoolListComponent`: Browse available privacy pools
- `HistoryComponent`: View saved deposit notes

#### UI Components

- `AssetSelector`: Choose from available assets
- `AmountInput`: Enter and validate amounts
- `NoteInput`: Load and parse deposit notes
- `NotificationContainer`: Display user notifications

### Services

#### ZKaneService

```rust
impl ZKaneService {
    pub async fn create_deposit(&self, asset_id: String, amount: u128) -> Result<DepositNote, ServiceError>;
    pub async fn generate_withdrawal_proof(&self, note: &DepositNote, outputs: &[TxOutput], merkle_path: &MerklePath) -> Result<WithdrawalProof, ServiceError>;
}
```

#### AlkanesService

```rust
impl AlkanesService {
    pub async fn get_user_assets(&self, address: &str) -> Result<Vec<AssetBalance>, ServiceError>;
    pub async fn get_privacy_pools(&self) -> Result<Vec<PoolInfo>, ServiceError>;
}
```

### Types

#### Core Types

```rust
pub struct DepositNote {
    pub asset_id: String,
    pub denomination: u128,
    pub commitment: String,
    pub nullifier: String,
    pub secret: String,
    pub leaf_index: u64,
    pub created_at: f64,
}

pub struct WithdrawalProof {
    pub proof: Vec<u8>,
    pub nullifier_hash: String,
    pub root: String,
    pub outputs: Vec<TxOutput>,
}
```

## Testing

### Running Tests

```bash
# Run all frontend tests
cargo test

# Run WASM tests in browser
wasm-pack test --headless --firefox

# Run component tests
cargo test frontend_component_tests

# Run integration tests
cargo test frontend_integration_tests
```

### Test Categories

1. **Unit Tests**: Individual function and component testing
2. **Component Tests**: UI component rendering and interaction
3. **Integration Tests**: Service integration and workflow testing
4. **WASM Tests**: Browser-specific functionality testing
5. **Performance Tests**: Rendering and creation benchmarks

## Configuration

### Environment Variables

- `ZKANE_API_URL`: Backend API URL (if using server mode)
- `ZKANE_NETWORK`: Bitcoin network (mainnet, testnet, regtest)
- `ZKANE_DEBUG`: Enable debug logging

### Build Configuration

```toml
[package.metadata.wasm-pack.profile.release]
wee-alloc = false

[package.metadata.wasm-pack.profile.dev]
debug-assertions = true
```

## Deployment

### Static Hosting

The application builds to static files that can be hosted on any HTTP server:

```bash
# Build for production
../../scripts/build-frontend.sh release

# Deploy dist/ directory to your hosting provider
rsync -av dist/ user@server:/var/www/zkane/
```

### IPFS Deployment

```bash
# Add to IPFS
ipfs add -r dist/

# Pin the hash
ipfs pin add <hash>
```

### CDN Deployment

The application works well with CDNs like Cloudflare, AWS CloudFront, or Netlify.

## Contributing

### Development Workflow

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/amazing-feature`
3. **Make changes**: Follow the coding standards
4. **Add tests**: Ensure new functionality is tested
5. **Run tests**: `../../scripts/test-all.sh frontend`
6. **Submit PR**: Create a pull request with description

### Coding Standards

- **Rust**: Follow `rustfmt` and `clippy` recommendations
- **Components**: Use functional components with hooks
- **Styling**: Use CSS variables for theming
- **Testing**: Write tests for all new functionality

### Architecture Guidelines

- **Separation of Concerns**: Keep UI, business logic, and data separate
- **Reactive Design**: Use signals and resources for state management
- **Error Handling**: Provide meaningful error messages to users
- **Performance**: Minimize WASM bundle size and optimize rendering

## Troubleshooting

### Common Issues

#### WASM Build Fails

```bash
# Ensure wasm-pack is up to date
cargo install wasm-pack --force

# Clear cache and rebuild
rm -rf pkg target
wasm-pack build --target web --dev
```

#### Application Won't Load

1. Check browser console for errors
2. Ensure HTTP server is serving from correct directory
3. Verify WASM files are being served with correct MIME type

#### Slow Performance

1. Build in release mode for production
2. Enable WASM optimization flags
3. Check for memory leaks in components

### Getting Help

- **Documentation**: Check the [Leptos Book](https://book.leptos.dev/)
- **Issues**: Report bugs on the GitHub repository
- **Discussions**: Join the community discussions
- **Discord**: Connect with other developers

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

## Acknowledgments

- **Leptos Team**: For the excellent web framework
- **Rust WASM Team**: For WebAssembly tooling
- **Alkanes Protocol**: For the asset infrastructure
- **ZKane Team**: For the privacy protocol design