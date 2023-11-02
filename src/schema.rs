// @generated automatically by Diesel CLI.

diesel::table! {
    accessories (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        description -> Nullable<Text>,
    }
}
