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
