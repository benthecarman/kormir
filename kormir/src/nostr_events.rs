#![cfg(feature = "nostr")]

use dlc_messages::oracle_msgs::{OracleAnnouncement, OracleAttestation};
use lightning::util::ser::Writeable;
use nostr::event::builder::Error;
use nostr::{Event, EventBuilder, EventId, Keys, Kind, Tag};

/// Creates an Oracle Attestation event for nostr.
pub fn create_announcement_event(
    keys: &Keys,
    announcement: &OracleAnnouncement,
    relays: &[String],
) -> Result<Event, Error> {
    let relays = relays.iter().map(|relay| relay.into()).collect::<Vec<_>>();
    let content = announcement.encode();
    EventBuilder::new(
        Kind::Custom(88),
        base64::encode(content),
        [Tag::Relays(relays)],
    )
    .to_event(keys)
}

/// Creates an Oracle Attestation event for nostr.
pub fn create_attestation_event(
    keys: &Keys,
    attestation: &OracleAttestation,
    event_id: EventId,
) -> Result<Event, Error> {
    let content = attestation.encode();
    EventBuilder::new(
        Kind::Custom(89),
        base64::encode(content),
        [Tag::Event {
            event_id,
            relay_url: None,
            marker: None,
        }],
    )
    .to_event(keys)
}
