#!/bin/bash

# ZKane Frontend Development Script
# This script starts the frontend development server with hot reloading using Trunk

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
FRONTEND_DIR="crates/zkane-frontend"
DEV_PORT="${1:-9080}"

echo -e "${BLUE}🚀 ZKane Frontend Development Server${NC}"
echo -e "${BLUE}====================================${NC}"

# Check prerequisites
echo -e "${YELLOW}📋 Checking prerequisites...${NC}"

# Check if trunk is installed
if ! command -v trunk &> /dev/null; then
    echo -e "${RED}❌ trunk is not installed${NC}"
    echo -e "${YELLOW}💡 Install with: cargo install trunk${NC}"
    exit 1
fi

# Check if cargo-watch is installed (optional but recommended)
if ! command -v cargo-watch &> /dev/null; then
    echo -e "${YELLOW}⚠️  cargo-watch not found, installing for better hot reloading...${NC}"
    cargo install cargo-watch
fi

echo -e "${GREEN}✅ Prerequisites check passed${NC}"

# Navigate to frontend directory
cd "$FRONTEND_DIR"

# Clean previous builds
if [ -d "dist" ]; then
    rm -rf dist
    echo -e "${GREEN}🧹 Cleaned previous build${NC}"
fi

echo -e "${YELLOW}🔥 Starting development server with hot reloading...${NC}"
echo -e "${GREEN}📱 Open http://localhost:${DEV_PORT} in your browser${NC}"
echo -e "${GREEN}🌐 External access: http://$(hostname):${DEV_PORT}${NC}"
echo -e "${YELLOW}🔄 Changes to src/, index.html, and styles.css will auto-reload${NC}"
echo -e "${YELLOW}Press Ctrl+C to stop the server${NC}"
echo ""

# Start development server with trunk
# Using cargo-watch for better file watching if available
if command -v cargo-watch &> /dev/null; then
    echo -e "${BLUE}🔍 Using cargo-watch for enhanced file watching${NC}"
    cargo-watch -w src -w index.html -w Trunk.toml -s "trunk serve --address 0.0.0.0 --port ${DEV_PORT}"
else
    echo -e "${BLUE}🔍 Using trunk's built-in file watching${NC}"
    trunk serve --address 0.0.0.0 --port "${DEV_PORT}"
fi