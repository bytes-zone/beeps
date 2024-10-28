#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"

(
    cd "${ROOT}/beeps-server"
    sqlx migrate run
)

"${ROOT}/scripts/dump-schema.sh"
