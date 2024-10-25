#!/usr/bin/env bash

ROOT="$(git rev-parse --show-toplevel)"

(
    cd "${ROOT}/beeps-server"
    sqlx migrate run
)

"${ROOT}/scripts/dump-schema.sh"
