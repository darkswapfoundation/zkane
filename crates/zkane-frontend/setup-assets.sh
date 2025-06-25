#!/bin/bash

# ZKane Frontend Asset Setup Script
# This script ensures the CSS file is properly copied to the dist/assets directory

echo "🚀 Setting up ZKane Frontend assets..."

# Create assets directory if it doesn't exist
mkdir -p dist/assets

# Copy CSS file
if [ -f "src/styles.css" ]; then
    cp src/styles.css dist/assets/
    echo "✅ CSS file copied to dist/assets/styles.css"
else
    echo "❌ Error: src/styles.css not found"
    exit 1
fi

# Copy favicon files
if [ -f "dist/assets/favicon.svg" ]; then
    cp dist/assets/favicon.svg dist/favicon.svg
    echo "✅ Favicon copied to root directory"
fi

# Check if CSS file was copied successfully
if [ -f "dist/assets/styles.css" ]; then
    echo "✅ Assets setup complete!"
    echo "📁 CSS file size: $(du -h dist/assets/styles.css | cut -f1)"
else
    echo "❌ Error: Failed to copy CSS file"
    exit 1
fi

echo ""
echo "🌐 Your ZKane frontend is ready!"
echo "📍 Server should be running at: http://localhost:9080"
echo "🎨 CSS available at: http://localhost:9080/assets/styles.css"
echo ""
echo "💡 If you see loading issues:"
echo "   1. Make sure this script ran successfully"
echo "   2. Check that the server is running: trunk serve --port 9080"
echo "   3. Verify CSS is accessible: curl http://localhost:9080/assets/styles.css"