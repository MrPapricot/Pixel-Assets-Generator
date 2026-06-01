#!/bin/bash
set -e

# Подстановка переменных из окружения
export PGPASSWORD="$POSTGRES_PASSWORD"

echo "!!! Creating user ${DB_USER}..."

# Создание пользователя
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    CREATE ROLE ${DB_USER} WITH LOGIN PASSWORD '${DB_USER_PASSWORD}';
    ALTER ROLE ${DB_USER} NOSUPERUSER;
    ALTER ROLE ${DB_USER} NOCREATEDB;
    ALTER ROLE ${DB_USER} NOCREATEROLE;
EOSQL

echo "!!! Granting user ${DB_USER} rights..."

# Предоставление прав
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    GRANT CONNECT ON DATABASE ${POSTGRES_DB} TO ${DB_USER};
    GRANT USAGE ON SCHEMA public TO ${DB_USER};
    GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO ${DB_USER};
    GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO ${DB_USER};

    -- Автоматическое предоставление прав на новые таблицы
    ALTER DEFAULT PRIVILEGES IN SCHEMA public
        GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO ${DB_USER};
    ALTER DEFAULT PRIVILEGES IN SCHEMA public
        GRANT USAGE, SELECT ON SEQUENCES TO ${DB_USER};

    -- Запрет опасных операций
    REVOKE CREATE ON SCHEMA public FROM ${DB_USER};
EOSQL

echo "!!! User ${DB_USER} successfully created"
