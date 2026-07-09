#!/usr/bin/env bash
# Fail if real `unsafe` keywords appear outside the documented allowlist.
# Matches: unsafe fn / unsafe impl / unsafe { / unsafe trait / ...
# Ignores the word "unsafe" inside comments and strings (best-effort: rg on code).
# See docs/UNSAFE_AUDIT_PLAN.md and docs/unsafe-code-guide.md.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Paths where unsafe is expected (substring match against file path).
ALLOW_GLOBS=(
  "src/ffi/"
  "src/vm/jit/"
  "src/vm/interpreter/safe_"
  "src/vm/interpreter/modules.rs"
  "src/objects/safe_typed_array.rs"
  "src/objects/strings.rs"
  "src/cli/build.rs"
  "modules/abi/"
  "modules/native-macros/"
  "benches/"
  "src/runtime_env/native_fns/json_fns.rs"
)

# Real unsafe keyword uses (not the English word in comments alone).
PATTERN='\bunsafe(\s+(fn|impl|trait|extern|const|static)|\s*\{)'

mapfile -t FILES < <(rg -l "$PATTERN" --type rust \
  -g '!target/**' -g '!node_modules/**' \
  src modules benches 2>/dev/null || true)

violations=0
for f in "${FILES[@]:-}"; do
  allowed=0
  for g in "${ALLOW_GLOBS[@]}"; do
    case "$f" in
      *"$g"*) allowed=1; break ;;
    esac
  done
  if [[ "$allowed" -eq 0 ]]; then
    echo "DISALLOWED unsafe in: $f"
    rg -n "$PATTERN" "$f" || true
    violations=$((violations + 1))
  fi
done

echo "---"
echo "Files with real unsafe keywords: ${#FILES[@]}"
if [[ "$violations" -gt 0 ]]; then
  echo "FAIL: $violations file(s) outside allowlist"
  exit 1
fi
echo "OK: all unsafe usages are within the allowlist"
