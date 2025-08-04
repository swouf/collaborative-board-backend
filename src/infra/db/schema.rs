// @generated automatically by Diesel CLI.

diesel::table! {
    updates (id) {
        id -> Int4,
        room_id -> Varchar,
        payload -> Bytea,
    }
}
