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

diesel::joinable!(adjustment -> adjustment_type (adjustment_type_id));

diesel::allow_tables_to_appear_in_same_query!(
    adjustment,
    adjustment_type,
);
