#!/usr/bin/env bash
set -euo pipefail

DATA="${1:-}"

if test -z "$DATA"; then
  echo "USAGE: ${0:-} path/to/data.json"
  exit 1
fi

NOW="$(python3 -c "from datetime import datetime; print(datetime.utcnow().isoformat(timespec='microseconds') + 'Z')")"

jq --arg NOW "$NOW" '. as $root | {timestamp: $NOW, counter: 0, node: 0} as $ts | $root.pings | map([{timestamp: $ts, op: {AddPing: {when: .time}}}, if .tag then {timestamp: $ts, op: {SetTag: {when: .time, tag: .tag}}} else empty end]) | add' "$DATA"
