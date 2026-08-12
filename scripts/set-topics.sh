#!/usr/bin/env bash
# Applies this repository's GitHub topics.
#
#   ./scripts/set-topics.sh                 # apply to the default repo
#   ./scripts/set-topics.sh owner/repo      # apply to a fork
#   ./scripts/set-topics.sh --print         # print the list, change nothing
#
# Topics are half of GitHub-side discovery: they drive repo search, the "explore
# topics" pages, and most third-party crawlers. They live here rather than only in
# someone's shell history so the list is reviewable and reappliable — GitHub has no
# concept of topics-as-code, so this script is the source of truth.
#
# Keep this list aligned with the "keywords" array in package.json. Same intent, two
# registries: npm search reads keywords, GitHub search reads topics.

set -euo pipefail
cd "$(dirname "$0")/.."

DEFAULT_REPO="Oakblu/plugin-react-data-testid-generator"

# GitHub rules: max 20 topics, each <= 50 chars, lowercase letters/numbers/hyphens only.
TOPICS=(
  swc
  swc-plugin
  react
  data-testid
  testid
  test-id
  e2e-testing
  end-to-end-testing
  playwright
  cypress
  testing-library
  test-automation
  jsx
  nextjs
  vite
  wasm
  rust
  codegen
  developer-tools
  testing
)

joined=$(IFS=,; echo "${TOPICS[*]}")

if [ "${1:-}" = "--print" ]; then
  echo "$joined"
  exit 0
fi

REPO="${1:-$DEFAULT_REPO}"

if [ "${#TOPICS[@]}" -gt 20 ]; then
  echo "error: GitHub allows at most 20 topics; this list has ${#TOPICS[@]}." >&2
  exit 1
fi

for t in "${TOPICS[@]}"; do
  if ! printf '%s' "$t" | grep -qE '^[a-z0-9][a-z0-9-]{0,49}$'; then
    echo "error: invalid topic '$t' — lowercase letters, digits and hyphens only." >&2
    exit 1
  fi
done

if ! command -v gh &>/dev/null; then
  echo "error: gh not found. Install the GitHub CLI (https://cli.github.com), or paste"
  echo "these into the repo's About panel on github.com:"
  echo
  echo "  $joined"
  exit 1
fi

if ! gh auth status &>/dev/null; then
  echo "error: gh is not authenticated. Run: gh auth login" >&2
  exit 1
fi

echo "Applying ${#TOPICS[@]} topics to $REPO:"
echo "  $joined"
gh repo edit "$REPO" --add-topic "$joined"
echo "Done. Verify: gh repo view $REPO --json repositoryTopics"
