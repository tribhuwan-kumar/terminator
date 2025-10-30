#!/bin/bash
# Demo: Dependency Caching Performance

set -e

echo "========================================="
echo "Dependency Caching Demo"
echo "========================================="
echo ""
echo "This demo shows how dependency caching improves workflow execution speed."
echo ""

VALID_DIR="C:/Users/louis/Documents/terminator/test-workflows/valid-workflow"
CACHE_DIR="C:/terminator-agent/.workflow-cache"

# Clean cache to start fresh
echo "Step 1: Cleaning cache to simulate first run..."
rm -rf "$CACHE_DIR"
echo "✓ Cache cleared"
echo ""

# First run - will install dependencies
echo "Step 2: First workflow execution (cache miss)..."
echo "This will:"
echo "  1. Detect package.json"
echo "  2. Calculate dependency hash"
echo "  3. Check cache (MISS)"
echo "  4. Run 'bun install' (slow)"
echo "  5. Copy node_modules to cache"
echo ""
START=$(date +%s)

# Simulate the dependency installation
cd "$VALID_DIR"
if command -v bun &> /dev/null; then
    echo "Running bun install..."
    bun install --frozen-lockfile 2>&1 | tail -5
else
    echo "Running npm install..."
    npm install --prefer-offline 2>&1 | tail -5
fi

END=$(date +%s)
FIRST_RUN_TIME=$((END - START))

echo ""
echo "✓ First run completed in ${FIRST_RUN_TIME}s"
echo ""

# Check cache was created
echo "Step 3: Verifying cache was created..."
if [ -d "$CACHE_DIR" ]; then
    echo "✓ Cache directory exists:"
    du -sh "$CACHE_DIR"
    echo ""
    echo "Cache structure:"
    ls -lh "$CACHE_DIR"
else
    echo "✗ Cache was not created"
    exit 1
fi
echo ""

# Second run - will use cache
echo "Step 4: Second workflow execution (cache hit)..."
echo "This will:"
echo "  1. Detect package.json"
echo "  2. Calculate dependency hash"
echo "  3. Check cache (HIT!)"
echo "  4. Copy cached node_modules (fast)"
echo ""

# Remove node_modules to simulate fresh execution
rm -rf "$VALID_DIR/node_modules"

START=$(date +%s)

# In actual implementation, this would be done by Rust code
# For demo, we simulate by copying from cache
HASH=$(find "$CACHE_DIR" -maxdepth 1 -type d | tail -1)
if [ -d "$HASH/node_modules" ]; then
    cp -r "$HASH/node_modules" "$VALID_DIR/"
    echo "✓ Using cached dependencies"
fi

END=$(date +%s)
SECOND_RUN_TIME=$((END - START))

echo ""
echo "✓ Second run completed in ${SECOND_RUN_TIME}s"
echo ""

# Calculate speedup
if [ $FIRST_RUN_TIME -gt 0 ]; then
    SPEEDUP=$((FIRST_RUN_TIME / SECOND_RUN_TIME))
    PERCENT=$((100 - (SECOND_RUN_TIME * 100 / FIRST_RUN_TIME)))
else
    SPEEDUP=0
    PERCENT=0
fi

echo "========================================="
echo "Results"
echo "========================================="
echo "First run (cache miss):  ${FIRST_RUN_TIME}s"
echo "Second run (cache hit):  ${SECOND_RUN_TIME}s"
echo "Speedup:                 ${SPEEDUP}x faster"
echo "Time saved:              ${PERCENT}%"
echo "========================================="
echo ""
echo "Key Benefits:"
echo "  ✓ Faster workflow execution on subsequent runs"
echo "  ✓ Reduced network bandwidth (no re-downloading packages)"
echo "  ✓ Persistent across workflow updates (if deps unchanged)"
echo "  ✓ Shared cache across workflows with same dependencies"
