// @generated automatically by Diesel CLI.

diesel::table! {
    pings (id) {
        id -> Integer,
        ping -> Timestamp,
    }
}
