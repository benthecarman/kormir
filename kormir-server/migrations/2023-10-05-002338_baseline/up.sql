-- Table of information about the oracle, mostly to prevent multiple keys from being used with the same database
-- singleton_constant is a dummy column to ensure there is only one row
CREATE TABLE oracle_metadata
(
    pubkey             bytea     NOT NULL UNIQUE PRIMARY KEY,
    name               TEXT      NOT NULL UNIQUE,
    created_at         timestamp NOT NULL DEFAULT NOW(),
    updated_at         timestamp NOT NULL DEFAULT NOW(),
    singleton_constant BOOLEAN   NOT NULL DEFAULT TRUE, -- make sure there is only one row
    CONSTRAINT one_row_check UNIQUE (singleton_constant)
);

-- Primary table containing information about events,
-- contains a broken up oracle announcement, excluding the oracle pubkey which is in memory
-- also contains the name of the event, and whether it is an enum or not for faster lookups
CREATE TABLE events
(
    id                     SERIAL PRIMARY KEY,
    announcement_signature bytea     NOT NULL,
    oracle_event           bytea     NOT NULL,
    name                   TEXT      NOT NULL UNIQUE,
    is_enum                BOOLEAN   NOT NULL,
    announcement_event_id  bytea UNIQUE,
    attestation_event_id   bytea UNIQUE,
    created_at             timestamp NOT NULL DEFAULT NOW(),
    updated_at             timestamp NOT NULL DEFAULT NOW()
);

-- index for faster lookups by name
CREATE UNIQUE INDEX event_name_index ON events (name);

-- Table for storing the nonces for each event
-- The signature and outcome are optional, and are only filled in when the event is completed
CREATE TABLE event_nonces
(
    id         INTEGER PRIMARY KEY,
    event_id   integer   NOT NULL REFERENCES events (id),
    index      INTEGER   NOT NULL,
    nonce      bytea     NOT NULL UNIQUE,
    signature  bytea,
    outcome    TEXT,
    created_at timestamp NOT NULL DEFAULT NOW(),
    updated_at timestamp NOT NULL DEFAULT NOW()
);

-- index for faster lookups by event_id
CREATE INDEX event_nonces_event_id_index ON event_nonces (event_id);
