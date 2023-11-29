// @generated automatically by Diesel CLI.

diesel::table! {
    adjustment (id) {
        id -> Unsigned<Bigint>,
        adjustment_type_id -> Unsigned<Bigint>,
        created -> Timestamp,
        #[max_length = 255]
        comment -> Nullable<Varchar>,
    }
}

diesel::table! {
    adjustment_type (id) {
        id -> Unsigned<Bigint>,
        #[max_length = 255]
        description -> Varchar,
        adjustment -> Tinyint,
    }
}

diesel::table! {
    time_entry (id) {
        id -> Unsigned<Bigint>,
        time -> Unsigned<Smallint>,
        created -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(adjustment, adjustment_type, time_entry,);
