#!/bin/bash

# ZKANE Native-Only Test Runner
# Purpose: Run tests without WASM compilation to avoid verbose linker dumps
# Prevents: WASM export symbol dumps, disk space issues, compilation timeouts

set -euo pipefail

echo "=== ZKANE Test Analysis & Solution ==="
echo "Problem identified:"
echo "1. Multi-target build (rlib + cdylib/WASM) causes verbose linker output"
echo "2. WASM compilation produces thousands of --export symbols"
echo "3. Full disk (100% -> 86% after cleanup) caused compilation failures"
echo "4. Failed compilation created corrupted bytecode dumps with ^^^^ symbols"
echo ""

# Check current disk usage
DISK_USAGE=$(df /home/e/Documents/zkane | tail -1 | awk '{print $5}' | sed 's/%//')
echo "Current disk usage: ${DISK_USAGE}%"
if [ "$DISK_USAGE" -gt 85 ]; then
    echo "WARNING: Consider 'cargo clean' if tests fail"
fi
echo ""

# Run tests with native target only to avoid WASM verbosity
OUTPUT_FILE="test_native_$(date +%Y%m%d_%H%M%S).txt"
CHAR_LIMIT=${1:-500}

echo "Running native-only tests (avoiding WASM compilation)..."
echo "Output limit: ${CHAR_LIMIT} characters"
echo "Output file: ${OUTPUT_FILE}"
echo ""

# Force native target only, skip WASM to avoid linker dump verbosity
timeout 120s cargo test --target aarch64-unknown-linux-gnu --lib 2>&1 | head -c "$CHAR_LIMIT" > "$OUTPUT_FILE" 2>&1 || {
    EXIT_CODE=$?
    echo "Exit code: $EXIT_CODE" >> "$OUTPUT_FILE"
    if [ $EXIT_CODE -eq 124 ]; then
        echo "Timed out after 120s" >> "$OUTPUT_FILE"
    fi
}

echo "Test complete!"
echo "First 20 lines of output:"
head -20 "$OUTPUT_FILE"
echo ""
echo "Full output saved to: $OUTPUT_FILE"