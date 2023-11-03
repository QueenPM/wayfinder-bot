use diesel::prelude::*;
use crate::schema::accessories;

// Accessory get

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::accessories)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Accessory {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub icon_emoji: Option<String>,
    pub tier: i32,
    pub url: String,
}

// Accessory set

#[derive(Insertable)]
#[diesel(table_name = accessories)]
pub struct NewAccessory<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub icon_url: &'a str,
    pub icon_emoji: &'a str,
    pub tier: i32,
    pub url: &'a str,
}