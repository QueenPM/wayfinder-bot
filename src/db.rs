use diesel::pg::PgConnection;
use diesel::prelude::*;

use dotenv::dotenv;
use std::env;

use diesel::{SelectableHelper, RunQueryDsl};

use crate::models::{NewAccessory, Accessory};

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn get_accessory(acessory_name: String) -> Result<Vec<Accessory>, String>{
    use crate::schema::accessories::dsl::*;

    let connection = &mut establish_connection();
    let results = accessories
        .limit(1)
        .filter(name.eq(acessory_name))
        .load(connection)
        .expect("Error loading accessories");
    println!("Displaying {} accessories", results.len());
    match results.len() {
        0 => Err("No accessories found".to_string()),
        _ => Ok(results),
    }
}

use diesel::result::Error as DieselError;

pub fn create_accessory(name: &str, description: &str) -> Result<Accessory, DieselError> {
    use crate::schema::accessories;
    let connection = &mut establish_connection();
    let new_accessory = NewAccessory { name, description };

    let result = diesel::insert_into(accessories::table)
        .values(&new_accessory)
        .returning(Accessory::as_returning())
        .get_result(connection);

    match result {
        Ok(accessory) => Ok(accessory),
        Err(e) => Err(e),
    }
}


pub async fn scrape_accessory(page:i32) -> Result<Vec<Accessory>, String>{
    // Get the URL
    let url = format!("https://www.studioloot.com/wayfinder/db/items/accessory/page/{}", page.to_string());
    // req that shit
    let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?.text().await.map_err(|e| e.to_string())?;
    // Get the document
    let document = scraper::Html::parse_document(&resp);
    // General selector
    let selector = scraper::Selector::parse(r#"body > div:nth-child(1) > div.flex.flex-col.min-h-screen > div > div.flex.flex-col.min-h-screen.bodybg-wayfinder > div > div.basis-3\/4.pl-0.md\:pl-4.shrink-0 > div > div.overflow-x-auto.box-shadow > table > tbody > tr"#).unwrap();
    // Get the children for accessories
    let children_vec: Vec<_> = document.select(&selector).collect();
    println!("Iterating for accessories, found {} accessories (children)", children_vec.len());
    // Iterate on that booya
    for (index, child) in children_vec.iter().enumerate() {
        let selector_string = format!("tr:nth-child({}) > td:nth-child(2) > a", index + 1);
        let name_selector = scraper::Selector::parse(&selector_string).unwrap();
        // TODO probably this doesnt have to be a vec but im not an expert scraper. As long as it works :shrug:
        let name_iterator: Vec<_> = child.select(&name_selector).map(|x| x.inner_html()).collect();
        match name_iterator.get(0) {
            Some(name) => {
                println!("Found item: {}", name);
            },
            None => {
                println!("No name found");
            }
        }
    }
    Err("This is just testing. Dont actually return anything".to_string())
}