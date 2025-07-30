// @generated automatically by Diesel CLI.

diesel::table! {
    updates (id) {
        id -> Integer,
        #[max_length = 255]
        room_id -> Varchar,
        payload -> Longblob,
    }
}
