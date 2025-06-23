#!/bin/bash

# ZKane Frontend Build Script
# This script builds the frontend application with WASM support

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
FRONTEND_DIR="crates/zkane-frontend"
BUILD_MODE="${1:-debug}"
SERVE_PORT="${2:-8080}"

echo -e "${BLUE}🏗️  ZKane Frontend Build Script${NC}"
echo -e "${BLUE}================================${NC}"

# Check prerequisites
echo -e "${YELLOW}📋 Checking prerequisites...${NC}"

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo -e "${RED}❌ wasm-pack is not installed${NC}"
    echo -e "${YELLOW}💡 Install with: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh${NC}"
    exit 1
fi

# Check if basic-http-server is installed (for serving)
if ! command -v basic-http-server &> /dev/null; then
    echo -e "${YELLOW}⚠️  basic-http-server not found, installing...${NC}"
    cargo install basic-http-server
fi

echo -e "${GREEN}✅ Prerequisites check passed${NC}"

# Build the WASM package
echo -e "${YELLOW}🔨 Building WASM package...${NC}"

cd "$FRONTEND_DIR"

# Clean previous builds
if [ -d "pkg" ]; then
    rm -rf pkg
    echo -e "${GREEN}🧹 Cleaned previous build${NC}"
fi

# Build with wasm-pack
if [ "$BUILD_MODE" = "release" ]; then
    echo -e "${YELLOW}🚀 Building in release mode...${NC}"
    wasm-pack build --target web --out-dir pkg --release --no-typescript
else
    echo -e "${YELLOW}🔧 Building in debug mode...${NC}"
    wasm-pack build --target web --out-dir pkg --dev --no-typescript
fi

echo -e "${GREEN}✅ WASM package built successfully${NC}"

# Copy additional assets
echo -e "${YELLOW}📁 Copying assets...${NC}"

# Create dist directory
mkdir -p dist

# Copy HTML file
cp index.html dist/

# Copy CSS file
cp src/styles.css dist/

# Copy WASM package
cp -r pkg dist/

# Create a simple favicon if it doesn't exist
if [ ! -f "dist/favicon.svg" ]; then
    cat > dist/favicon.svg << 'EOF'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
  <circle cx="50" cy="50" r="40" fill="#007bff"/>
  <text x="50" y="60" text-anchor="middle" fill="white" font-family="Arial" font-size="30" font-weight="bold">Z</text>
</svg>
EOF
    echo -e "${GREEN}📄 Created favicon${NC}"
fi

echo -e "${GREEN}✅ Assets copied successfully${NC}"

# Build summary
echo -e "${BLUE}📊 Build Summary${NC}"
echo -e "${BLUE}===============${NC}"
echo -e "Mode: ${BUILD_MODE}"
echo -e "Output directory: ${FRONTEND_DIR}/dist"
echo -e "WASM package size: $(du -h pkg/zkane_frontend_bg.wasm | cut -f1)"

# Offer to serve the application
echo -e "${YELLOW}🌐 Would you like to serve the application? (y/n)${NC}"
read -r serve_choice

if [ "$serve_choice" = "y" ] || [ "$serve_choice" = "Y" ]; then
    echo -e "${YELLOW}🚀 Starting development server on port ${SERVE_PORT}...${NC}"
    echo -e "${GREEN}📱 Open http://localhost:${SERVE_PORT} in your browser${NC}"
    echo -e "${YELLOW}Press Ctrl+C to stop the server${NC}"
    
    cd dist
    basic-http-server --addr 127.0.0.1:${SERVE_PORT}
else
    echo -e "${GREEN}✅ Build complete! Serve the 'dist' directory with any HTTP server.${NC}"
    echo -e "${BLUE}💡 Example: cd ${FRONTEND_DIR}/dist && basic-http-server${NC}"
fi