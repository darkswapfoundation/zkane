#!/bin/bash

# ZKane Comprehensive Test Script
# This script runs all tests including unit, integration, WASM, and frontend tests

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
TEST_MODE="${1:-all}"
VERBOSE="${2:-false}"

echo -e "${BLUE}🧪 ZKane Comprehensive Test Suite${NC}"
echo -e "${BLUE}==================================${NC}"

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to run a test and track results
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo -e "${YELLOW}🔍 Running: ${test_name}${NC}"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if [ "$VERBOSE" = "true" ]; then
        echo -e "${BLUE}Command: ${test_command}${NC}"
    fi
    
    if eval "$test_command"; then
        echo -e "${GREEN}✅ PASSED: ${test_name}${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}❌ FAILED: ${test_name}${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    echo ""
}

# Check prerequisites
echo -e "${YELLOW}📋 Checking prerequisites...${NC}"

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo -e "${RED}❌ wasm-pack is not installed${NC}"
    echo -e "${YELLOW}💡 Install with: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh${NC}"
    exit 1
fi

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ cargo is not installed${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Prerequisites check passed${NC}"
echo ""

# Core Rust tests
if [ "$TEST_MODE" = "all" ] || [ "$TEST_MODE" = "core" ]; then
    echo -e "${BLUE}🦀 Running Core Rust Tests${NC}"
    echo -e "${BLUE}=========================${NC}"
    
    run_test "ZKane Core Tests" "cargo test --package zkane-core"
    run_test "ZKane Crypto Tests" "cargo test --package zkane-crypto"
    run_test "ZKane Common Tests" "cargo test --package zkane-common"
    run_test "ZKane WASM Tests" "cargo test --package zkane-wasm"
    run_test "Main Workspace Tests" "cargo test --lib"
fi

# WASM-specific tests
if [ "$TEST_MODE" = "all" ] || [ "$TEST_MODE" = "wasm" ]; then
    echo -e "${BLUE}🌐 Running WASM Tests${NC}"
    echo -e "${BLUE}===================${NC}"
    
    # Build WASM packages first
    run_test "Build ZKane WASM Package" "cd crates/zkane-wasm && wasm-pack build --target web --dev"
    run_test "Build Frontend WASM Package" "cd crates/zkane-frontend && wasm-pack build --target web --dev"
    
    # Run WASM tests in browser
    run_test "WASM Integration Tests" "cd crates/zkane-wasm && wasm-pack test --headless --firefox"
    run_test "Frontend Component Tests" "cd crates/zkane-frontend && wasm-pack test --headless --firefox"
fi

# Alkanes tests
if [ "$TEST_MODE" = "all" ] || [ "$TEST_MODE" = "alkanes" ]; then
    echo -e "${BLUE}⚗️  Running Alkanes Tests${NC}"
    echo -e "${BLUE}========================${NC}"
    
    run_test "ZKane Alkanes Tests" "cargo test --manifest-path alkanes/zkane/Cargo.toml"
    run_test "ZKane Factory Tests" "cargo test --manifest-path alkanes/zkane-factory/Cargo.toml"
fi

# Noir circuit tests
if [ "$TEST_MODE" = "all" ] || [ "$TEST_MODE" = "noir" ]; then
    echo -e "${BLUE}🔮 Running Noir Circuit Tests${NC}"
    echo -e "${BLUE}=============================${NC}"
    
    # Check if nargo is installed
    if command -v nargo &> /dev/null; then
        run_test "Noir Withdraw Circuit Tests" "cd noir/withdraw && nargo test"
    else
        echo -e "${YELLOW}⚠️  nargo not found, skipping Noir tests${NC}"
        echo -e "${YELLOW}💡 Install Noir: curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash${NC}"
    fi
fi

# Integration tests
if [ "$TEST_MODE" = "all" ] || [ "$TEST_MODE" = "integration" ]; then
    echo -e "${BLUE}🔗 Running Integration Tests${NC}"
    echo -e "${BLUE}============================${NC}"
    
    run_test "End-to-End Flow Tests" "cargo test end_to_end_flow_tests"
    run_test "WASM Integration Tests" "cargo test wasm_integration"
    run_test "Frontend Integration Tests" "cargo test frontend_integration_tests"
fi

# Linting and formatting
if [ "$TEST_MODE" = "all" ] || [ "$TEST_MODE" = "lint" ]; then
    echo -e "${BLUE}🧹 Running Linting and Formatting${NC}"
    echo -e "${BLUE}=================================${NC}"
    
    run_test "Cargo Format Check" "cargo fmt -- --check"
    run_test "Cargo Clippy" "cargo clippy --all-targets --all-features -- -D warnings"
    run_test "Cargo Check" "cargo check --all-targets --all-features"
fi

# Documentation tests
if [ "$TEST_MODE" = "all" ] || [ "$TEST_MODE" = "docs" ]; then
    echo -e "${BLUE}📚 Running Documentation Tests${NC}"
    echo -e "${BLUE}==============================${NC}"
    
    run_test "Documentation Tests" "cargo test --doc"
    run_test "Documentation Build" "cargo doc --no-deps --all-features"
fi

# Security audit
if [ "$TEST_MODE" = "all" ] || [ "$TEST_MODE" = "security" ]; then
    echo -e "${BLUE}🔒 Running Security Audit${NC}"
    echo -e "${BLUE}=========================${NC}"
    
    # Check if cargo-audit is installed
    if command -v cargo-audit &> /dev/null; then
        run_test "Security Audit" "cargo audit"
    else
        echo -e "${YELLOW}⚠️  cargo-audit not found, installing...${NC}"
        if cargo install cargo-audit; then
            run_test "Security Audit" "cargo audit"
        else
            echo -e "${RED}❌ Failed to install cargo-audit${NC}"
            FAILED_TESTS=$((FAILED_TESTS + 1))
            TOTAL_TESTS=$((TOTAL_TESTS + 1))
        fi
    fi
fi

# Performance benchmarks
if [ "$TEST_MODE" = "all" ] || [ "$TEST_MODE" = "bench" ]; then
    echo -e "${BLUE}⚡ Running Performance Benchmarks${NC}"
    echo -e "${BLUE}=================================${NC}"
    
    run_test "Crypto Benchmarks" "cargo bench --package zkane-crypto"
    run_test "Core Benchmarks" "cargo bench --package zkane-core"
fi

# Test Summary
echo -e "${BLUE}📊 Test Summary${NC}"
echo -e "${BLUE}===============${NC}"
echo -e "Total Tests: ${TOTAL_TESTS}"
echo -e "${GREEN}Passed: ${PASSED_TESTS}${NC}"
echo -e "${RED}Failed: ${FAILED_TESTS}${NC}"

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}🎉 All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}💥 Some tests failed!${NC}"
    exit 1
fi