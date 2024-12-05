-- First modify the foreign key table (event_nonces)
ALTER TABLE event_nonces
DROP CONSTRAINT event_nonces_event_id_fkey;

-- Add new column to events
ALTER TABLE events 
ADD COLUMN event_id TEXT;

-- Copy data
UPDATE events 
SET event_id = CONCAT('EVENT_', id::text);

-- Add new column to event_nonces
ALTER TABLE event_nonces 
ADD COLUMN new_event_id TEXT;

-- Copy data in event_nonces
UPDATE event_nonces 
SET new_event_id = CONCAT('EVENT_', event_id::text);

-- Now we can safely modify the primary table
ALTER TABLE events 
DROP CONSTRAINT events_pkey;

ALTER TABLE events 
DROP COLUMN id;

ALTER TABLE events 
ADD PRIMARY KEY (event_id);

-- Finally update event_nonces
ALTER TABLE event_nonces 
DROP COLUMN event_id;

ALTER TABLE event_nonces 
RENAME COLUMN new_event_id TO event_id;

-- Re-add the foreign key constraint
ALTER TABLE event_nonces
ADD CONSTRAINT event_nonces_event_id_fkey
FOREIGN KEY (event_id) REFERENCES events(event_id);