// @generated automatically by Diesel CLI.

diesel::table! {
    minutes_per_pings (id) {
        id -> Integer,
        minutes_per_ping -> Integer,
        timestamp -> TimestamptzSqlite,
        counter -> Integer,
        node -> Integer,
    }
}

diesel::table! {
    pings (id) {
        id -> Integer,
        ping -> TimestamptzSqlite,
    }
}

diesel::table! {
    tags (id) {
        id -> Integer,
        ping -> TimestamptzSqlite,
        tag -> Nullable<Text>,
        timestamp -> TimestamptzSqlite,
        counter -> Integer,
        node -> Integer,
    }
}

diesel::allow_tables_to_appear_in_same_query!(minutes_per_pings, pings, tags,);
