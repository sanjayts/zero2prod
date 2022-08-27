-- Add migration script here
CREATE TABLE subscriptions(
    id uuid NOT NULL PRIMARY KEY,
    email TEXT NOT NULL UNIQUE ,
    name TEXT NOT NULL UNIQUE ,
    subscribed_at timestamptz NOT NULL
);