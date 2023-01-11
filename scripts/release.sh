#!/bin/bash
# Usage: ./scripts/release.sh <major|minor|patch>
set -ex

if [ $# -ne 1 ]; then
  echo "Usage: ./release.sh <major|minor|patch>"
  exit 1
fi

# Fail if there are any unstaged changes left
git diff --exit-code

cargo workspaces version $1
NEW_VERSION=$(cargo workspaces ls --json | jq -r '.[] | select(.name == "codspeed") | .version')
cargo workspaces publish --from-git
gh release create v$NEW_VERSION --title "v$NEW_VERSION" --generate-notes -d
