#!/usr/bin/env bash
set -euo pipefail

if test -z "$PGDATA"; then
    echo '$PGDATA is not set. Use direnv or set it manually.'
    exit 1
fi

if test -z "$PGUSER"; then
    echo '$PGUSER is not set. Use direnv or set it manually.'
    exit 1
fi

start() {
    pg_ctl start -l "$PGDATA/logfile"
}


if ! test -d "$PGDATA"; then
    initdb --username "$PGUSER"
    start
    createdb "$PGUSER"
else
    start
fi
