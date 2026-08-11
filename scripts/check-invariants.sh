#!/usr/bin/env bash
# Repo invariants that are easy to violate silently. Run from the repository root.
#
#   ./scripts/check-invariants.sh          # report violations, exit 1 if any
#   ./scripts/check-invariants.sh --fix    # auto-fix what is safely fixable, then re-check
#
# Every check prints PASS/FAIL plus the exact command to fix it. Checks are ordered
# cheapest-first so a failure surfaces fast.

set -uo pipefail
cd "$(dirname "$0")/.."

FIX=false
[ "${1:-}" = "--fix" ] && FIX=true

FAILED=0
pass() { printf '  \033[32mPASS\033[0m  %s\n' "$1"; }
fail() { printf '  \033[31mFAIL\033[0m  %s\n     ↳ fix: %s\n' "$1" "$2"; FAILED=$((FAILED + 1)); }

echo "Repo invariants"

# ── 1. Version parity: Cargo.toml package version == package.json version ──────
PKG_V=$(node -p "require('./package.json').version" 2>/dev/null || echo "?")
CARGO_V=$(awk '/^version = / { gsub(/"/, "", $3); print $3; exit }' Cargo.toml)
if [ "$PKG_V" = "$CARGO_V" ]; then
  pass "version parity (package.json $PKG_V == Cargo.toml $CARGO_V)"
elif [ "$FIX" = true ]; then
  awk -v v="$PKG_V" '!d && /^version = / { sub(/"[^"]*"/, "\"" v "\""); d=1 } { print }' \
    Cargo.toml > Cargo.toml.tmp && mv Cargo.toml.tmp Cargo.toml
  cargo update -p swc_plugin_react_data_testid >/dev/null 2>&1
  pass "version parity — auto-fixed Cargo.toml to $PKG_V"
else
  fail "version drift (package.json $PKG_V != Cargo.toml $CARGO_V)" \
       "./scripts/check-invariants.sh --fix"
fi

# ── 2. No untracked test fixtures ──────────────────────────────────────────────
UNTRACKED_FIX=$(git ls-files --others --exclude-standard -- tests/ 2>/dev/null)
if [ -z "$UNTRACKED_FIX" ]; then
  pass "all test fixtures tracked by git"
elif [ "$FIX" = true ]; then
  git add tests/ && pass "fixtures — auto-staged: $(echo "$UNTRACKED_FIX" | tr '\n' ' ')"
else
  fail "untracked fixtures: $(echo "$UNTRACKED_FIX" | tr '\n' ' ')" "git add tests/"
fi

# ── 3. No silently-discarded fallible calls in the plugin ──────────────────────
# `let _ = <fs/io call>` hides failures from users. The plugin runs in a WASI sandbox
# where filesystem calls always fail, so swallowing them produces silent no-ops.
# Matches both `std::fs::…` and the aliased `fs::…` form (after `use std::fs;`).
# `serde_json::from_str(..).ok()` is the same failure in config form: deserialization is
# all-or-nothing, so one mistyped option discards every option the user set, silently.
SWALLOWED=$(grep -nE 'let _ = (std::)?fs::|from_str\([^)]*\)\.ok\(\)' src/lib.rs || true)
if [ -z "$SWALLOWED" ]; then
  pass "no swallowed fs or config-parse results in src/lib.rs"
else
  fail "swallowed errors: $SWALLOWED" \
       "handle the Result and call report_error() so users see the failure"
fi

# ── 4. Wasm artifact is not stale relative to plugin source ───────────────────
if [ ! -f plugin.wasm ]; then
  pass "plugin.wasm absent (built on demand by npm run build)"
elif [ plugin.wasm -nt src/lib.rs ] && [ plugin.wasm -nt Cargo.toml ]; then
  pass "plugin.wasm newer than src/lib.rs and Cargo.toml"
elif [ "$FIX" = true ]; then
  npm run build >/dev/null 2>&1 && pass "plugin.wasm — auto-rebuilt"
else
  fail "plugin.wasm is stale — examples would test an old binary" "npm run build"
fi

# ── 5. Every fixture has an output — i.e. it actually ran ─────────────────────
# `#[testing::fixture("tests/fixture/*/input.tsx")]` expands its glob at COMPILE time.
# A fixture directory added without touching tests/fixture.rs is silently never run:
# cargo reuses the cached test binary and the suite still reports all green. A missing
# output.tsx is the visible symptom.
ORPHANS=""
while IFS= read -r input; do
  [ -f "$(dirname "$input")/output.tsx" ] || ORPHANS="$ORPHANS $input"
done < <(find tests -mindepth 3 -name input.tsx 2>/dev/null)
if [ -z "$ORPHANS" ]; then
  pass "every fixture input.tsx has an output.tsx (all fixtures ran)"
elif [ "$FIX" = true ]; then
  # Judge success by whether the outputs appeared, not by the exit code: on a first
  # generation the assertions that read output.tsx can run before the fixture that
  # writes it, so UPDATE=1 legitimately exits non-zero while doing the right thing.
  touch tests/fixture.rs
  UPDATE=1 cargo test >/dev/null 2>&1
  STILL_MISSING=""
  for input in $ORPHANS; do
    [ -f "$(dirname "$input")/output.tsx" ] || STILL_MISSING="$STILL_MISSING $input"
  done
  if [ -z "$STILL_MISSING" ]; then
    git add tests/ >/dev/null 2>&1
    pass "fixtures — auto-generated missing outputs for:$ORPHANS"
  else
    fail "fixtures with no output.tsx:$STILL_MISSING" \
         "touch tests/fixture.rs && UPDATE=1 cargo test"
  fi
else
  fail "fixtures never ran (no output.tsx):$ORPHANS" \
       "touch tests/fixture.rs && UPDATE=1 cargo test"
fi

# ── 6. Wasm target installed (cargo test alone cannot catch link errors) ───────
if rustup target list --installed 2>/dev/null | grep -q wasm32-wasip1; then
  pass "wasm32-wasip1 target installed"
else
  fail "wasm32-wasip1 target missing — the WASM link step cannot run" \
       "rustup target add wasm32-wasip1"
fi

echo
if [ "$FAILED" -eq 0 ]; then
  echo "All invariants hold."
else
  echo "$FAILED invariant(s) violated. Re-run with --fix to auto-fix where possible."
fi
exit "$FAILED"
