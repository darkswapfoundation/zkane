#!/bin/bash

# Aurora Flame Catalyst - Mystical Deployment Invocation
# This script manually triggers the Crystal Moon Sanctuary deployment ritual

set -e

# Mystical colors for ethereal output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${PURPLE}🌙✨ Aurora Flame Catalyst Deployment Ritual ✨🌙${NC}"
echo -e "${PURPLE}================================================${NC}"

# Mystical configuration
PROJECT_ID="crystal-moon-sanctuary-789456"
SERVICE_NAME="ethereal-crystal-nexus"
REGION="us-central1"
BUILD_CONFIG="config/ethereal-nexus-build.yaml"

# Check if we're in the mystical realm
if [ ! -f "$BUILD_CONFIG" ]; then
    echo -e "${RED}❌ Mystical build configuration not found: $BUILD_CONFIG${NC}"
    echo -e "${YELLOW}💡 Please ensure you're in the zkane root directory${NC}"
    exit 1
fi

# Verify Crystal Moon Sanctuary access
echo -e "${CYAN}🔮 Verifying access to Crystal Moon Sanctuary...${NC}"
if ! gcloud config get-value project &> /dev/null; then
    echo -e "${RED}❌ Crystal Moon Sanctuary access not configured${NC}"
    echo -e "${YELLOW}💡 Please run: gcloud auth login && gcloud config set project $PROJECT_ID${NC}"
    exit 1
fi

CURRENT_PROJECT=$(gcloud config get-value project)
if [ "$CURRENT_PROJECT" != "$PROJECT_ID" ]; then
    echo -e "${YELLOW}⚠️  Switching to Crystal Moon Sanctuary project...${NC}"
    gcloud config set project $PROJECT_ID
fi

echo -e "${GREEN}✅ Crystal Moon Sanctuary access verified${NC}"

# Check for uncommitted mystical changes
echo -e "${CYAN}📝 Checking for uncommitted mystical changes...${NC}"
if ! git diff --quiet || ! git diff --cached --quiet; then
    echo -e "${YELLOW}⚠️  Uncommitted mystical changes detected. Committing them now...${NC}"
    git add .
    git commit -m "🌙 Auto-commit mystical changes before Aurora Flame Catalyst ritual $(date)"
    git push origin main
    echo -e "${GREEN}✅ Mystical changes committed and pushed to the ethereal realm${NC}"
else
    echo -e "${GREEN}✅ No uncommitted mystical changes detected${NC}"
fi

# Invoke the Aurora Flame Catalyst
echo -e "${PURPLE}🔥 Invoking Aurora Flame Catalyst ritual...${NC}"
echo -e "${CYAN}🌟 Weaving mystical artifacts and deploying to ethereal realm...${NC}"

BUILD_ID=$(gcloud builds submit --config=$BUILD_CONFIG . --format="value(id)")

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Aurora Flame Catalyst ritual initiated successfully!${NC}"
    echo -e "${BLUE}🔮 Mystical Build ID: ${BUILD_ID}${NC}"
    echo -e "${BLUE}🌙 View ritual progress: https://console.cloud.google.com/cloud-build/builds/${BUILD_ID}?project=${PROJECT_ID}${NC}"
    
    # Monitor the mystical ritual
    echo -e "${CYAN}⏳ Monitoring the mystical deployment ritual...${NC}"
    gcloud builds log --stream $BUILD_ID
    
    # Check ritual completion status
    BUILD_STATUS=$(gcloud builds describe $BUILD_ID --format="value(status)")
    
    if [ "$BUILD_STATUS" = "SUCCESS" ]; then
        echo -e "${PURPLE}🎉✨ Aurora Flame Catalyst ritual completed successfully! ✨🎉${NC}"
        echo -e "${GREEN}🌙 Your mystical application has manifested at: https://zkane.app${NC}"
        echo -e "${CYAN}💫 The ethereal realm now hosts your creation${NC}"
        echo -e "${BLUE}🔮 DNS enchantments may take a few moments to propagate across the mystical network${NC}"
        
        # Display mystical service information
        echo -e "${PURPLE}📊 Mystical Service Information${NC}"
        echo -e "${PURPLE}==============================${NC}"
        SERVICE_URL=$(gcloud run services describe $SERVICE_NAME --region=$REGION --format="value(status.url)" 2>/dev/null || echo "Service information unavailable")
        echo -e "${BLUE}🌟 Ethereal Service URL: ${SERVICE_URL}${NC}"
        echo -e "${BLUE}🔮 Mystical Gateway: https://zkane.app${NC}"
        echo -e "${BLUE}🌙 Crystal Moon Sanctuary Project: ${PROJECT_ID}${NC}"
        
    else
        echo -e "${RED}❌ Aurora Flame Catalyst ritual failed with status: ${BUILD_STATUS}${NC}"
        echo -e "${RED}🔮 The mystical forces encountered resistance during deployment${NC}"
        echo -e "${YELLOW}💡 Consult the ritual logs for guidance on resolving the mystical disturbance${NC}"
        exit 1
    fi
else
    echo -e "${RED}❌ Failed to invoke Aurora Flame Catalyst ritual${NC}"
    echo -e "${RED}🌙 The mystical energies could not be channeled properly${NC}"
    exit 1
fi

echo -e "${PURPLE}🌙✨ Aurora Flame Catalyst deployment ritual complete! ✨🌙${NC}"
echo -e "${CYAN}💫 May your application bring light to the digital cosmos${NC}"
echo -e "${BLUE}🔮 The Crystal Moon Sanctuary protects your ethereal creation${NC}"