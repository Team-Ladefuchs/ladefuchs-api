#!/bin/sh

if [ ! -f ./config.env ]; then
    echo "config.env is missing - copy config.env.example and point DATABASE_URL at the local Postgres container" >&2
    exit 1
fi

set -a
source ./config.env
set +a

sqlx database create || true
sqlx migrate run

cargo test --features testing
