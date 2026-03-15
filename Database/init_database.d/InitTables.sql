\connect TestDB

CREATE TABLE users (
    uuid uuid PRIMARY KEY DEFAULT uuidv7(),
    username text NOT NULL,
    password_hash text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);