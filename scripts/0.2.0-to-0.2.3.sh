#!/usr/bin/env bash
set -euo pipefail

DATA="${1:-}"

if test -z "$DATA"; then
  echo "USAGE: ${0:-} path/to/data.json"
  exit 1
fi

jq '{ ops: . }' "$DATA"
