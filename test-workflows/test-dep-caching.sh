#!/bin/bash
# Test dependency caching implementation

set -e

echo "========================================="
echo "Testing TypeScript Workflow Dependency Caching"
echo "========================================="

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test directories
TEST_DIR="C:/Users/louis/Documents/terminator/test-workflows"
VALID_DIR="$TEST_DIR/valid-workflow"
INVALID_MULTIPLE="$TEST_DIR/invalid-multiple-workflows"
INVALID_NO_TERMINATOR="$TEST_DIR/invalid-no-terminator"

echo ""
echo "Test 1: Valid workflow with terminator.ts"
echo "----------------------------------------"
if [ -f "$VALID_DIR/terminator.ts" ]; then
    echo -e "${GREEN}✓${NC} terminator.ts exists"
else
    echo -e "${RED}✗${NC} terminator.ts missing"
    exit 1
fi

if [ -f "$VALID_DIR/package.json" ]; then
    echo -e "${GREEN}✓${NC} package.json exists"
else
    echo -e "${RED}✗${NC} package.json missing"
    exit 1
fi

echo ""
echo "Test 2: Invalid workflow with multiple workflow files"
echo "----------------------------------------------------"
if [ -f "$INVALID_MULTIPLE/terminator.ts" ] && [ -f "$INVALID_MULTIPLE/my-workflow.ts" ]; then
    echo -e "${GREEN}✓${NC} Multiple workflow files present (should be rejected)"
else
    echo -e "${RED}✗${NC} Test setup incorrect"
    exit 1
fi

echo ""
echo "Test 3: Invalid workflow without terminator.ts"
echo "----------------------------------------------"
if [ ! -f "$INVALID_NO_TERMINATOR/terminator.ts" ] && [ -f "$INVALID_NO_TERMINATOR/finance-data-entry.ts" ]; then
    echo -e "${GREEN}✓${NC} No terminator.ts (should be rejected)"
else
    echo -e "${RED}✗${NC} Test setup incorrect"
    exit 1
fi

echo ""
echo "Test 4: Check cache directory structure"
echo "---------------------------------------"
CACHE_DIR="C:/terminator-agent/.workflow-cache"
if [ -d "$CACHE_DIR" ]; then
    echo -e "${YELLOW}ℹ${NC}  Cache directory exists: $CACHE_DIR"
    echo "Cache contents:"
    ls -la "$CACHE_DIR" 2>/dev/null || echo "  (empty)"
else
    echo -e "${YELLOW}ℹ${NC}  Cache directory will be created on first workflow run"
fi

echo ""
echo "========================================="
echo "All setup tests passed!"
echo "========================================="
echo ""
echo "To test end-to-end workflow execution:"
echo "  1. Run unit tests: cargo test --package terminator-mcp-agent workflow_typescript::tests"
echo "  2. Test valid workflow should install deps and cache them"
echo "  3. Second run should use cached deps (much faster)"
