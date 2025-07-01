# ZKane Technical Context

## Technology Stack

### Frontend Framework
- **Leptos**: Modern reactive web framework for Rust
- **WebAssembly (WASM)**: Compiled Rust code running in browser
- **Trunk**: Build tool for WASM compilation and development server
- **leptos-router**: Client-side routing
- **leptos-meta**: HTML head management

### Cryptography
- **Noir**: Zero-knowledge proof language and circuits
- **Poseidon Hash**: Cryptographic hash function optimized for ZK proofs
- **Merkle Trees**: For privacy pool commitment structures

### Styling & UI
- **Custom CSS**: 28KB stylesheet with CSS variables
- **Responsive Design**: Mobile-first approach
- **Theme System**: Light/Dark/Auto themes
- **CSS Grid & Flexbox**: Modern layout techniques

### Development Tools
- **Rust 1.88.0**: Latest stable Rust compiler
- **Cargo**: Package manager and build system
- **wasm-bindgen**: Rust-WASM bindings
- **wasm-bindgen-test**: Browser-based testing

## Build Configuration

### Trunk.toml
```toml
[build]
target = "index.html"
dist = "dist"

[serve]
port = 9080
```

### Cargo Workspace Structure
```
zkane/
├── crates/
│   ├── zkane-frontend/     # Main frontend application
│   ├── zkane-core/         # Core business logic
│   ├── zkane-crypto/       # Cryptographic primitives
│   └── zkane-common/       # Shared types and utilities
├── alkanes/                # Alkanes protocol integration
└── noir/                   # Zero-knowledge circuits
```

## Development Workflow

### Local Development
1. `trunk serve --port 9080` - Start development server
2. Hot reload enabled for rapid iteration
3. WASM compilation on file changes
4. Asset management via `setup-assets.sh`

### Testing Strategy
- **Unit Tests**: Rust-based component testing
- **Integration Tests**: WASM browser testing
- **E2E Tests**: Full application workflow testing

## Performance Considerations

### WASM Optimization
- Optimized build size for web delivery
- Lazy loading of cryptographic components
- Efficient memory management

### CSS Optimization
- CSS variables for theme switching
- Minimal external dependencies
- Optimized for mobile performance

## Browser Compatibility
- **Modern Browsers**: Chrome 57+, Firefox 52+, Safari 11+
- **WASM Support**: Required for application functionality
- **LocalStorage**: For persistent user data
- **ES6+ Features**: Modern JavaScript support

## Security Considerations
- **Client-Side Only**: No server-side data exposure
- **Local Storage Encryption**: Sensitive data protection
- **Zero-Knowledge Proofs**: Cryptographic privacy guarantees
- **No External Dependencies**: Reduced attack surface