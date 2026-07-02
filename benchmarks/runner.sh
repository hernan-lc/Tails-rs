#!/usr/bin/env bash
set -euo pipefail

RUNTIMES=(node bun deno tails)
WARMUP="${WARMUP:-2}"
RUNS="${RUNS:-5}"
# Per-run wall-clock timeout in seconds. Default 2m; set TIMEOUT=60 for 1m.
# A run exceeding this is killed (exit 124) and recorded as a failed "timeout"
# rather than blocking the whole suite — so an optimization plan can start even
# if one benchmark hangs.
TIMEOUT="${TIMEOUT:-120}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUT="$SCRIPT_DIR/results/latest.json"
TS=$(date +%Y%m%d_%H%M%S)
mkdir -p "$SCRIPT_DIR/results/runs"

TAILS_BIN=""
if command -v tails &>/dev/null; then
  TAILS_BIN="tails"
elif command -v cargo &>/dev/null; then
  TAILS_BIN="cargo run --quiet --release --bin tails --"
fi

TMP_JSONL="$(mktemp)"
trap 'rm -f "$TMP_JSONL"' EXIT

for script in "$ROOT"/benchmarks/suites/*/*.js; do
  [[ -f "$script" ]] || continue
  script_name="$(basename "$script")"
  suite_name="$(basename "$(dirname "$script")")"
  script_id="$suite_name/$script_name"

  for runtime in "${RUNTIMES[@]}"; do
    runtime_label="$runtime"
    available=false
    if command -v "$runtime" &>/dev/null; then
      available=true
    elif [[ "$runtime" == "tails" ]]; then
      if [[ -n "$TAILS_BIN" ]]; then
        available=true
      fi
    fi

    if ! $available; then
      echo "SKIP $runtime $script_id (not in PATH)" >&2
      printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
        "$script_id" "$runtime_label" "$suite_name" 0 0 "" '{"skipped":true,"reason":"not in PATH"}' >> "$TMP_JSONL"
      continue
    fi

    times=()
    failed=0
    last_error=""
    skipped=false

    for i in $(seq 1 $((WARMUP + RUNS))); do
      out=""
      rc=0
      set +e
      if [[ "$runtime" == "tails" ]]; then
        out=$(timeout "$TIMEOUT" $TAILS_BIN run "$script" 2>/dev/null)
        rc=$?
      elif [[ "$runtime" == "deno" ]]; then
        out=$(timeout "$TIMEOUT" deno run --allow-read --allow-net "$script" 2>/dev/null)
        rc=$?
      elif [[ "$runtime" == "bun" ]]; then
        out=$(timeout "$TIMEOUT" bun run "$script" 2>/dev/null)
        rc=$?
      else
        out=$(timeout "$TIMEOUT" "$runtime" "$script" 2>/dev/null)
        rc=$?
      fi
      set -e

      # 124 = `timeout` killed the process for exceeding TIMEOUT seconds
      if [[ $rc -eq 124 ]]; then
        failed=$((failed + 1))
        last_error="timeout after ${TIMEOUT}s"
        continue
      fi
      if [[ "$out" == "SKIP"* ]]; then
        skipped=true
        break
      fi
      if [[ $rc -ne 0 ]] || [[ -z "$out" ]]; then
        failed=$((failed + 1))
        last_error="runtime exited non-zero (rc=$rc)"
        continue
      fi
      first_line="${out%%$'\n'*}"
      if [[ "$first_line" =~ ^[0-9]+\.?[0-9]*$ ]]; then
        if [[ $i -gt $WARMUP ]]; then
          times+=("$first_line")
        fi
      else
        failed=$((failed + 1))
        last_error="unexpected first line: $first_line"
      fi
    done

    if $skipped; then
      printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
        "$script_id" "$runtime_label" "$suite_name" 0 0 "" '{"skipped":true}' >> "$TMP_JSONL"
      echo "SKIP $runtime $script_id" >&2
      continue
    fi

    if [[ ${#times[@]} -eq 0 ]]; then
      printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
        "$script_id" "$runtime_label" "$suite_name" 0 "$failed" "$last_error" "{}" >> "$TMP_JSONL"
      echo "FAIL $runtime $script_id ($failed failures)" >&2
      continue
    fi

    stats=$(printf '%s\n' "${times[@]}" | node "$ROOT/benchmarks/tools/average.js")
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
      "$script_id" "$runtime_label" "$suite_name" "${#times[@]}" "$failed" "" "$stats" >> "$TMP_JSONL"

    mean_us=$(echo "$stats" | python3 -c 'import json,sys; print(json.load(sys.stdin)["mean_us"])')
    stdev_us=$(echo "$stats" | python3 -c 'import json,sys; print(json.load(sys.stdin)["stdev_us"])')
    n=$(echo "$stats" | python3 -c 'import json,sys; print(json.load(sys.stdin)["n"])')
    echo "OK  $runtime $script_id (mean=${mean_us}us stdev=${stdev_us}us n=$n)" >&2
  done
done

python3 "$SCRIPT_DIR/tools/aggregate.py" "$TMP_JSONL" "$OUT" "$SCRIPT_DIR/results/runs/${TS}.json"
node "$SCRIPT_DIR/tools/report.js" "$OUT" "$SCRIPT_DIR/results/REPORT.md"
echo "Benchmark complete. Report: $SCRIPT_DIR/results/REPORT.md"
