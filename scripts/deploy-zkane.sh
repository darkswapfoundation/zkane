#!/bin/bash

# ZKane Deployment Script
# This script manually triggers the deployment pipeline

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 ZKane Deployment Pipeline${NC}"
echo -e "${BLUE}=============================${NC}"

# Check if we're in the right directory
if [ ! -f "cloudbuild.yaml" ]; then
    echo -e "${RED}❌ cloudbuild.yaml not found. Please run this script from the zkane directory.${NC}"
    exit 1
fi

# Check if gcloud is configured
if ! gcloud config get-value project &> /dev/null; then
    echo -e "${RED}❌ gcloud is not configured. Please run 'gcloud auth login' and 'gcloud config set project PROJECT_ID'${NC}"
    exit 1
fi

PROJECT_ID=$(gcloud config get-value project)
echo -e "${YELLOW}📋 Project ID: ${PROJECT_ID}${NC}"

# Commit and push changes if there are any
echo -e "${YELLOW}📝 Checking for uncommitted changes...${NC}"
if ! git diff --quiet || ! git diff --cached --quiet; then
    echo -e "${YELLOW}⚠️  You have uncommitted changes. Committing them now...${NC}"
    git add .
    git commit -m "Auto-commit before deployment $(date)"
    git push origin main
    echo -e "${GREEN}✅ Changes committed and pushed${NC}"
else
    echo -e "${GREEN}✅ No uncommitted changes${NC}"
fi

# Trigger Cloud Build manually
echo -e "${YELLOW}🔨 Triggering Cloud Build...${NC}"
BUILD_ID=$(gcloud builds submit --config=cloudbuild.yaml . --format="value(id)")

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Build submitted successfully!${NC}"
    echo -e "${BLUE}📊 Build ID: ${BUILD_ID}${NC}"
    echo -e "${BLUE}🔗 View build logs: https://console.cloud.google.com/cloud-build/builds/${BUILD_ID}?project=${PROJECT_ID}${NC}"
    
    # Wait for build to complete
    echo -e "${YELLOW}⏳ Waiting for build to complete...${NC}"
    gcloud builds log --stream $BUILD_ID
    
    # Check build status
    BUILD_STATUS=$(gcloud builds describe $BUILD_ID --format="value(status)")
    
    if [ "$BUILD_STATUS" = "SUCCESS" ]; then
        echo -e "${GREEN}🎉 Deployment completed successfully!${NC}"
        echo -e "${GREEN}🌐 Your app should be available at: https://zkane.app${NC}"
        echo -e "${BLUE}💡 It may take a few minutes for DNS changes to propagate.${NC}"
    else
        echo -e "${RED}❌ Build failed with status: ${BUILD_STATUS}${NC}"
        echo -e "${RED}🔗 Check the build logs for details.${NC}"
        exit 1
    fi
else
    echo -e "${RED}❌ Failed to submit build${NC}"
    exit 1
fi

echo -e "${BLUE}✨ Deployment pipeline completed!${NC}"