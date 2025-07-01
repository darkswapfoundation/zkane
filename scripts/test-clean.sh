#!/bin/bash

# ZKane Clean Test Runner - Aligned with boiler framework
# Purpose: Run tests without WASM compilation or bytecode generation
# Prevents: Massive hex dumps, ^^^ symbols, verbose linker output

set -euo pipefail

# Colors for clean output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🧪 ZKane Clean Test Suite (Boiler-Aligned)${NC}"
echo -e "${BLUE}===========================================${NC}"

# BOILER PATTERN: Multiple environment variables to ensure ZERO bytecode generation
export ZKANE_SKIP_BUILD=1
export CARGO_CFG_TEST=1
export RUST_TEST_THREADS=1
export ZKANE_TEST_MODE=1
export CARGO_TARGET_DIR="target/test-clean"

# Test configuration
TEST_TARGET="${1:-all}"
CHAR_LIMIT="${2:-2000}"  # Increased for clean output
TIMEOUT="${3:-120}"      # Increased timeout for thorough testing

echo -e "${YELLOW}📋 Configuration:${NC}"
echo "  Target: $TEST_TARGET"
echo "  Output limit: $CHAR_LIMIT chars"
echo "  Timeout: ${TIMEOUT}s"
echo "  Bytecode generation: DISABLED"
echo ""

# Function to run test with clean output
run_clean_test() {
    local test_name="$1"
    local test_cmd="$2"
    
    echo -e "${YELLOW}🔍 Running: ${test_name}${NC}"
    
    # Create output file with timestamp
    local output_file="test_${test_name}_$(date +%Y%m%d_%H%M%S).log"
    
    # Run test with limits to prevent log overflow
    if timeout "${TIMEOUT}s" $test_cmd 2>&1 | head -c "$CHAR_LIMIT" > "$output_file"; then
        echo -e "${GREEN}✅ PASSED: ${test_name}${NC}"
        
        # Show clean summary (first few lines only)
        echo "Summary:"
        head -5 "$output_file" | grep -E "(test result|running|passed)" || echo "  Test completed successfully"
    else
        echo -e "${RED}❌ FAILED: ${test_name}${NC}"
        echo "Error details:"
        tail -10 "$output_file"
    fi
    
    echo "Full log: $output_file"
    echo ""
}

# BOILER PATTERN: Native-only test execution (ZERO WASM compilation)
case "$TEST_TARGET" in
    "all")
        run_clean_test "core" "cargo test --features test-clean --lib"
        run_clean_test "zkane_withdrawal" "cargo test --features test-clean zkane_withdrawal_verification"
        run_clean_test "comprehensive" "cargo test --features test-clean comprehensive_zkane"
        ;;
    "core")
        run_clean_test "core_only" "cargo test --features test-clean --lib"
        ;;
    "withdrawal")
        run_clean_test "withdrawal_only" "cargo test --features test-clean zkane_withdrawal_verification"
        ;;
    "comprehensive")
        run_clean_test "comprehensive_only" "cargo test --features test-clean comprehensive_zkane"
        ;;
    "quick")
        # Ultra-fast native test with minimal output
        run_clean_test "quick_native" "cargo test --features test-clean --lib clean_output_verification"
        ;;
    *)
        echo -e "${RED}Unknown test target: $TEST_TARGET${NC}"
        echo "Available targets: all, core, withdrawal, comprehensive, quick"
        exit 1
        ;;
esac

echo -e "${BLUE}📊 Clean Testing Complete${NC}"
echo -e "${GREEN}No bytecode dumps, no ^^^ symbols, clean output! 🎉${NC}"