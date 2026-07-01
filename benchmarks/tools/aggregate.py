#!/usr/bin/env python3
import json, shutil, datetime, sys

path = sys.argv[1]
out_path = sys.argv[2]
archive_path = sys.argv[3]

entries = []
with open(path) as f:
  for line in f:
    line = line.rstrip('\n')
    if not line:
      continue
    parts = line.split('\t')
    if len(parts) != 7:
      continue
    script_id, runtime, suite, runs_completed, runs_failed, error, stats_json = parts
    entry = {
      "script": script_id,
      "runtime": runtime,
      "suite": suite,
      "runs_completed": int(runs_completed),
      "runs_failed": int(runs_failed),
      "error": error if error else None,
    }
    try:
      stats = json.loads(stats_json)
      entry.update(stats)
    except Exception:
      entry["skipped"] = True
    entries.append(entry)

result = {
  "generated": datetime.datetime.now(datetime.timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ'),
  "entries": entries,
}
with open(out_path, 'w') as f:
  json.dump(result, f, indent=2)

shutil.copy(out_path, archive_path)
print(f"Results written to {out_path}")
print(f"Archived as {archive_path}")
