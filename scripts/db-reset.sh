#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"

(
    cd "${ROOT}/beeps-server"
    sqlx database reset
)

"${ROOT}/scripts/dump-schema.sh"
