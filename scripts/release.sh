#!/bin/bash
# Usage: ./scripts/release.sh <major|minor|patch>
set -ex

if [ $# -ne 1 ]; then
  echo "Usage: ./release.sh <major|minor|patch>"
  exit 1
fi

# Fail if not on main branch
git branch --show-current | grep -q '^main$'
# Fail if there are any unstaged changes left
git diff --exit-code

cargo workspaces version --no-individual-tags --exact --no-git-push --force "*" $1
git push --follow-tags
