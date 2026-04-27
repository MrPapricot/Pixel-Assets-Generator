#!/bin/bash
set -e

    export PGPASSWORD="$POSTGRES_PASSWORD"

    echo "!!! Creating tables in  ${DB_NAME}..."

    psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL

CREATE TABLE IF NOT EXISTS users (
                                     uuid uuid PRIMARY KEY DEFAULT uuidv7(),
                                     email text NOT NULL UNIQUE,
                                     password_hash text NOT NULL,
                                     created_at timestamptz NOT NULL DEFAULT now()
);
EOSQL

echo "!!! All tables successfully created"
