#!/bin/bash

# 🚀 Push and synchronize with GitHub
# This script ensures your local branch is up-to-date and pushes both code and version tags.
# Change to the project root directory
cd "$(dirname "$0")/.."

VERSION="0.6.0"

# 🔄 Synchronization & Rebase
echo "📦 Checking for unstaged changes..."
HAS_STASH=false
if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "📥 Auto-stashing your local changes..."
  git stash push -u -m "Auto-stash by zz_upload.sh $(date)"
  HAS_STASH=true
fi

# echo "🔄 Pulling from remote and rebasing..."
# git pull --rebase origin main

if [ "$HAS_STASH" = true ]; then
  echo "📤 Restoring your local changes..."
  git stash pop
fi

# 🧙 Version Synchronization Magic
# Ensure we only use the numeric part for configuration files
CLEAN_VERSION="${VERSION#v}"
echo "🧙 Synchronizing version v${CLEAN_VERSION} to all configuration files..."

# 1. Update pyproject.toml
if [[ "$OSTYPE" == "darwin"* ]]; then
  sed -i '' "s/^version = \".*\"/version = \"$CLEAN_VERSION\"/" pyproject.toml
else
  sed -i "s/^version = \".*\"/version = \"$CLEAN_VERSION\"/" pyproject.toml
fi

# 2. Update create-wheels.sh
if [[ "$OSTYPE" == "darwin"* ]]; then
  sed -i '' "s/^VERSION=\".*\"/VERSION=\"$CLEAN_VERSION\"/" helpers/create-wheels.sh
else
  sed -i "s/^VERSION=\".*\"/VERSION=\"$CLEAN_VERSION\"/" helpers/create-wheels.sh
fi

# 3. Update src/src-tauri/Cargo.toml
if [[ "$OSTYPE" == "darwin"* ]]; then
  sed -i '' "s/^version = \".*\"/version = \"$CLEAN_VERSION\"/" src/src-tauri/Cargo.toml
else
  sed -i "s/^version = \".*\"/version = \"$CLEAN_VERSION\"/" src/src-tauri/Cargo.toml
fi

# 4. Check if a tag for this version already exists
if git rev-parse "v$CLEAN_VERSION" >/dev/null 2>&1; then
  echo "🏷 Tag v$CLEAN_VERSION already exists. Skipping tag creation."
else
  echo "🏷 Creating new tag v$CLEAN_VERSION..."
  # Commit the version bump if there are changes
  git add pyproject.toml helpers/create-wheels.sh src/src-tauri/Cargo.toml
  git commit -m "🚀 build: synchronize version v$CLEAN_VERSION" || echo "No changes to commit for version bump."
  git tag "v$CLEAN_VERSION"
fi

echo "🚀 Pushing current branch to GitHub (FORCE)..."
git push origin main --force

echo "🏷 Pushing version tags (FORCE)..."
git push origin --tags --force

# 🧹 Clean up old tags (keep only latest 3)
echo "🧹 Cleaning up older tags (keeping latest 3)..."
# Get all tags except the 3 most recent ones (sorted by version)
TAGS_TO_DELETE=$(git tag -l --sort=-v:refname | tail -n +4)

if [ -n "$TAGS_TO_DELETE" ]; then
  for TAG in $TAGS_TO_DELETE; do
    echo "🗑 Deleting old tag: $TAG"
    git tag -d "$TAG"
    git push origin --delete "$TAG" || echo "Tag $TAG already removed from remote."
  done
else
  echo "✅ No old tags to delete."
fi

echo "✨ Sync and Publish complete for v${CLEAN_VERSION}!"
