-- First remove the foreign key constraint
ALTER TABLE event_nonces 
DROP CONSTRAINT event_nonces_event_id_fkey;

-- Add old integer columns back
ALTER TABLE events 
ADD COLUMN id INT4;

ALTER TABLE event_nonces 
ADD COLUMN new_event_id INT4;

-- Generate sequential IDs (you might want to modify this logic based on your needs)
WITH numbered_rows AS (
  SELECT event_id, ROW_NUMBER() OVER (ORDER BY created_at) as rnum
  FROM events
)
UPDATE events e
SET id = n.rnum
FROM numbered_rows n
WHERE e.event_id = n.event_id;

-- Update the foreign key references
UPDATE event_nonces en
SET new_event_id = e.id
FROM events e
WHERE en.event_id = e.event_id;

-- Make id primary key again
ALTER TABLE events 
DROP CONSTRAINT events_pkey;

ALTER TABLE events 
ADD PRIMARY KEY (id);

-- Clean up the text columns
ALTER TABLE events 
DROP COLUMN event_id;

ALTER TABLE event_nonces 
DROP COLUMN event_id;

ALTER TABLE event_nonces 
RENAME COLUMN new_event_id TO event_id;

-- Re-add the foreign key constraint
ALTER TABLE event_nonces
ADD CONSTRAINT event_nonces_event_id_fkey
FOREIGN KEY (event_id) REFERENCES events(id);