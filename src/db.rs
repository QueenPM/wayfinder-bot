use diesel::pg::PgConnection;
use diesel::prelude::*;

use dotenv::dotenv;
use serenity::futures::{stream, StreamExt};
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
    let new_accessory = NewAccessory { 
        name, description,
        icon_url: "no",
        tier: 0,
        icon_emoji: "no",
        url: "no"
    };

    let result = diesel::insert_into(accessories::table)
        .values(&new_accessory)
        .returning(Accessory::as_returning())
        .get_result(connection);

    match result {
        Ok(accessory) => Ok(accessory),
        Err(e) => Err(e),
    }
}

fn scrape_dom(selector_string: &str, html_string: &str) -> Option<String> {
    let document = scraper::Html::parse_document(html_string);

    let selector = match scraper::Selector::parse(&selector_string) {
        Ok(selector) => selector,
        Err(_) => return None,
    };

    let html_vector: Vec<_> = document.select(&selector).map(|x| x.inner_html()).collect();

    match html_vector.get(0) {
        Some(html) => Some(html.clone()),
        None => None,
    }
}

pub async fn scrape_all() -> Result<String,String>{
    let result = scrape_accessory_page(1).await;
    match result {
        Ok(_) => Ok("Successfully scraped all accessories".to_string()),
        Err(e) => Err(e),
    }
}

pub async fn scrape_accessory_page(page:i32) -> Result<Vec<Accessory>, String>{
    let client = reqwest::Client::new();

    let page_url = format!("https://www.studioloot.com/wayfinder/db/items/accessory/page/{}", page.to_string());
    let page = client.get(&page_url).send().await.map_err(|e| e.to_string())?.text().await.map_err(|e| e.to_string())?;

    let mut accessory_urls:Vec<String> = Vec::new();

    let result = async {
        let document = scraper::Html::parse_document(&page);
        let accessories_selector = scraper::Selector::parse("tbody").unwrap();
        let tbodies: Vec<_> = document.select(&accessories_selector).map(|x| x.inner_html()).collect();
        let tbody = tbodies.get(0).ok_or("No tbody found");
        if tbody.is_err() {
            return Err("No tbody found".to_string());
        }
        let accessory_selector = scraper::Selector::parse(":nth-child(2) > a").unwrap();
        let tbody_dom = scraper::Html::parse_document(tbody.unwrap());
        accessory_urls = tbody_dom.select(&accessory_selector)
            .filter_map(|x| x.attr("href"))
            .map(|href| format!("https://studioloot.com{}", href))
            .collect();
        Ok(())
    }.await;

    if result.is_err(){
        return Err(result.err().unwrap().to_string());
    }

    let bodies = stream::iter(accessory_urls)
        .map(|url| {
            let client = &client;
            async move {
                let resp = client.get(&url).send().await.map_err(|e| e.to_string())?.text().await.map_err(|e| e.to_string());
                resp
            }
        })
        .buffer_unordered(1);

    bodies.for_each(|b| async move {
        let accessory_page = match b {
            Ok(page) => page,
            Err(e) => {
                eprintln!("Failed to get accessory page: {}", e);
                return;
            }
        };

        let name = scrape_dom("h1", &accessory_page).ok_or_else(|| "No name found".to_string());
        let description = scrape_dom(".text-wf-secondary", &accessory_page).ok_or_else(|| "No description found".to_string());
        let tier = scrape_dom(".font-saira.tracking-wider.uppercase.bold.italic.mx-2.brightness-125", &accessory_page).ok_or_else(|| "No tier found".to_string());

        // Scrape the attributes
        // Find the attributes div containing all attributes
        let attributes_selector = scraper::Selector::parse(".block.gap-x-16.flex-wrap.mx-auto.w-fit").unwrap();
        let attributes_dom = scraper::Html::parse_document(&accessory_page);
        let attributes = attributes_dom.select(&attributes_selector).map(|x| x.inner_html()).collect::<Vec<_>>();
        let attributes = attributes.get(0).ok_or_else(|| "No attributes found".to_string());
        if attributes.is_err() {
            eprintln!("Failed to scrape accessory details");
            return;
        }
        // Get all the child attributes (actual attributes) into a vector
        let attributes = scraper::Html::parse_document(attributes.unwrap());
        let attributes_selector = scraper::Selector::parse(".w-32.h-32.relative").unwrap();
        let attributes = attributes.select(&attributes_selector).map(|x| x.inner_html()).collect::<Vec<_>>();
        println!("Found {} attributes", attributes.len());
        for attribute in attributes {
            let attribute_name = scrape_dom(":nth-child(2) > :nth-child(2) > :nth-child(1)", &attribute).ok_or_else(|| "No attribute selector found".to_string());
            let attribute_value = scrape_dom(":nth-child(2) > :nth-child(2) > :nth-child(2)", &attribute).ok_or_else(|| "No attribute selector found".to_string());

            // Attributes
            match (attribute_name, attribute_value) {
                (Ok(attribute_name), Ok(attribute_value)) => {
                    // remove <br> at the end of attribute_name
                    let attribute_name = attribute_name.replace("<br>", "");
                    println!("{}: {}", attribute_name, attribute_value);
                }
                _ => {
                    eprintln!("Failed to scrape accessory details");
                }
            }
        }

        // Check for the set bonus
        // TODO

        match (name, description, tier) {
            (Ok(name), Ok(description), Ok(tier)) => {
                println!("Got accessory {} ({}) {}", name, tier, description);
            }
            _ => {
                eprintln!("Failed to scrape accessory details");
            }
        }
    }).await;

    Err("This is just testing. Don't actually return anything".to_string())
}