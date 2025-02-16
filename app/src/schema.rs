// @generated automatically by Diesel CLI.

diesel::table! {
    pings (id) {
        id -> Integer,
        ping -> Timestamp,
    }
}

diesel::table! {
    tags (id) {
        id -> Integer,
        ping -> Timestamp,
        tag -> Nullable<Text>,
        timestamp -> Timestamp,
        counter -> Integer,
        node -> Integer,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    pings,
    tags,
);
