#!/bin/bash

# ZKANE Safe Test Runner
# Purpose: Run cargo test with output limits to prevent LLM crashes from verbose dumps
# Addresses: Disk space issues, WASM linker verbosity, bytecode corruption

set -euo pipefail

# Check disk space first
DISK_USAGE=$(df /home/e/Documents/zkane | tail -1 | awk '{print $5}' | sed 's/%//')
if [ "$DISK_USAGE" -gt 90 ]; then
    echo "WARNING: Disk usage is ${DISK_USAGE}%. Consider running 'cargo clean' first."
    echo "Current space:"
    df -h /home/e/Documents/zkane
fi

# Output file with timestamp
OUTPUT_FILE="test_output_$(date +%Y%m%d_%H%M%S).txt"
CHAR_LIMIT=${1:-500}  # Default 500 chars, can be overridden

echo "Running cargo test with ${CHAR_LIMIT} character limit..."
echo "Output will be saved to: ${OUTPUT_FILE}"

# Run test with proper limits and redirection
timeout 120s cargo test --verbose 2>&1 | head -c "$CHAR_LIMIT" > "$OUTPUT_FILE" 2>&1 || {
    EXIT_CODE=$?
    echo "Test exit code: $EXIT_CODE" >> "$OUTPUT_FILE"
    if [ $EXIT_CODE -eq 124 ]; then
        echo "Test timed out after 120 seconds" >> "$OUTPUT_FILE"
    fi
}

echo "Test complete. Output saved to $OUTPUT_FILE"
echo "First few lines:"
head -10 "$OUTPUT_FILE"