#!/bin/bash
# Recovery script after force push incident
# Run this to sync with the cleaned main branch

echo "==================================="
echo "Terminator Repository Recovery Tool"
echo "==================================="
echo ""

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "❌ Error: Not in a git repository!"
    exit 1
fi

# Check current branch
CURRENT_BRANCH=$(git branch --show-current)
echo "📍 Current branch: $CURRENT_BRANCH"
echo ""

# Check for uncommitted changes
if ! git diff-index --quiet HEAD -- 2>/dev/null; then
    echo "⚠️  You have uncommitted changes!"
    echo "   Please commit or stash them first:"
    echo "   git stash"
    echo ""
    read -p "Do you want to stash them now? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git stash
        echo "✅ Changes stashed"
    else
        echo "❌ Aborting. Please handle your changes first."
        exit 1
    fi
fi

# Check if there are unpushed commits
UNPUSHED=$(git log origin/main..HEAD --oneline 2>/dev/null)
if [ ! -z "$UNPUSHED" ]; then
    echo "⚠️  You have unpushed commits:"
    echo "$UNPUSHED"
    echo ""
    echo "Creating backup branch: backup-$(date +%Y%m%d-%H%M%S)"
    BACKUP_BRANCH="backup-$(date +%Y%m%d-%H%M%S)"
    git branch $BACKUP_BRANCH
    echo "✅ Your commits are saved in branch: $BACKUP_BRANCH"
    echo ""
fi

# Fetch latest from origin
echo "📥 Fetching latest from GitHub..."
git fetch origin

# Reset to origin/main
echo "🔄 Resetting to origin/main..."
git reset --hard origin/main

echo ""
echo "✅ SUCCESS! Your repository is now synced with the cleaned main branch."
echo ""
echo "Current version: $(grep '^version' Cargo.toml | head -1)"
echo ""

if [ ! -z "$BACKUP_BRANCH" ]; then
    echo "📌 Your previous commits are saved in: $BACKUP_BRANCH"
    echo "   To see them: git log $BACKUP_BRANCH"
    echo "   To cherry-pick a commit: git cherry-pick <commit-hash>"
fi

if [ ! -z "$(git stash list)" ]; then
    echo ""
    echo "📌 Don't forget to restore your stashed changes:"
    echo "   git stash pop"
fi

echo ""
echo "You can now continue working normally! 🎉"