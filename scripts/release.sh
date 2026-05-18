#!/usr/bin/env bash
set -euo pipefail

# Usage: ./scripts/release.sh [patch|minor|major]  (default: patch)
#        ./scripts/release.sh --dry-run [patch|minor|major]
#
# Must be run from the repository root.

BUMP="patch"
DRY_RUN=false

for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=true ;;
    patch|minor|major) BUMP="$arg" ;;
    *) echo "Usage: $0 [--dry-run] [patch|minor|major]"; exit 1 ;;
  esac
done

# ── Preflight ──────────────────────────────────────────────────────────────────

if ! command -v cargo &>/dev/null; then
  echo "error: cargo not found. Install Rust: https://rustup.rs"
  exit 1
fi

if ! rustup target list --installed | grep -q "wasm32-wasip1"; then
  echo "error: wasm32-wasip1 target not installed."
  echo "  Run: rustup target add wasm32-wasip1"
  exit 1
fi

if ! npm whoami &>/dev/null 2>&1; then
  echo "error: not logged in to npm. Run: npm login"
  exit 1
fi

CURRENT_VERSION=$(node -p "require('./package.json').version")
echo "Current version : $CURRENT_VERSION"
echo "Bump type       : $BUMP"
echo "Dry run         : $DRY_RUN"
echo ""

# ── Tests ──────────────────────────────────────────────────────────────────────

echo "▶ Running cargo tests..."
cargo test
echo ""

# ── Build ──────────────────────────────────────────────────────────────────────

echo "▶ Building WASM..."
cargo build --target wasm32-wasip1 --release
cp target/wasm32-wasip1/release/swc_plugin_react_data_testid.wasm plugin.wasm
echo "  plugin.wasm: $(du -h plugin.wasm | cut -f1)"
echo ""

# ── Version bump ───────────────────────────────────────────────────────────────

NEW_VERSION=$(npm version "$BUMP" --no-git-tag-version)
NEW_VERSION="${NEW_VERSION#v}"
echo "▶ Version bumped: $CURRENT_VERSION → $NEW_VERSION"
echo ""

if [ "$DRY_RUN" = true ]; then
  echo "Dry run — stopping before publish."
  echo "Would publish: swc-plugin-react-data-testid@$NEW_VERSION"
  # Restore the version bump
  npm version "$CURRENT_VERSION" --no-git-tag-version --allow-same-version &>/dev/null
  exit 0
fi

# ── Publish ────────────────────────────────────────────────────────────────────

echo "▶ Publishing to npm..."
npm publish --access public
echo ""

# ── Git tag ────────────────────────────────────────────────────────────────────

TAG="swc-v${NEW_VERSION}"

git add package.json
git commit -m "chore: bump swc-plugin version to v${NEW_VERSION}"
git tag -a "$TAG" -m "swc-plugin-react-data-testid v${NEW_VERSION}"

echo "▶ Created commit and tag: $TAG"
echo ""
echo "Done. Push with:"
echo "  git push && git push origin $TAG"
