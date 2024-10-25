#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
SCHEMA="${ROOT}/beeps-server/test/schema.sql"

pg_dump --schema-only --exclude-schema _sqlx_test > "$SCHEMA"
