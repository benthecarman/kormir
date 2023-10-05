// @generated automatically by Diesel CLI.

diesel::table! {
    event_nonces (id) {
        id -> Int4,
        event_id -> Int4,
        index -> Int4,
        nonce -> Bytea,
        signature -> Nullable<Bytea>,
        outcome -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    events (id) {
        id -> Int4,
        announcement_signature -> Bytea,
        oracle_event -> Bytea,
        name -> Text,
        is_enum -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(event_nonces -> events (event_id));

diesel::allow_tables_to_appear_in_same_query!(
    event_nonces,
    events,
);
