// @generated automatically by Diesel CLI.

diesel::table! {
    adjustment_type (id) {
        id -> Unsigned<Bigint>,
        #[max_length = 255]
        name -> Varchar,
        adjustment -> Tinyint,
    }
}
