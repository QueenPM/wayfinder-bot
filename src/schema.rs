// @generated automatically by Diesel CLI.

diesel::table! {
    accessories (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        description -> Nullable<Text>,
        #[max_length = 255]
        icon_url -> Nullable<Varchar>,
        #[max_length = 255]
        icon_emoji -> Nullable<Varchar>,
        tier -> Int4,
        #[max_length = 255]
        url -> Varchar,
    }
}

diesel::table! {
    accessory_levels (id) {
        id -> Int4,
        accessory_id -> Int4,
        max_health -> Nullable<Int4>,
        resillience -> Nullable<Int4>,
        weapon_power -> Nullable<Int4>,
        ability_power -> Nullable<Int4>,
        crit_rating -> Nullable<Int4>,
        crit_power -> Nullable<Int4>,
        break_power -> Nullable<Int4>,
        phys_defense -> Nullable<Int4>,
        mag_defense -> Nullable<Int4>,
    }
}

diesel::joinable!(accessory_levels -> accessories (accessory_id));

diesel::allow_tables_to_appear_in_same_query!(
    accessories,
    accessory_levels,
);
