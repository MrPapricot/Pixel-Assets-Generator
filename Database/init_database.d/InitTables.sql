\connect TestDB

CREATE TABLE users (
    uuid uuid PRIMARY KEY DEFAULT uuidv7(),
    email text NOT NULL UNIQUE,
    password_hash text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);