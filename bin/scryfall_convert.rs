use rusqlite::{params, Result};
use serde_json::Value;
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Set up the database
    let mut conn = mtg::db::init()?;

    // Step 2: Read the JSON file
    let file = File::open("./data/scryfall-default-cards.json")?;
    let reader = BufReader::new(file);
    let cards: Vec<Value> = serde_json::from_reader(reader)?;

    // Step 3: Insert data into the database
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(
            "INSERT OR REPLACE INTO cards (
                id, name, set_code, collector_number, type_line, oracle_text,
                mana_cost, cmc, colors, rarity, image_uris
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )?;

        for card in cards {
            stmt.execute(params![
                card["id"].as_str(),
                card["name"].as_str(),
                card["set"].as_str(),
                card["collector_number"].as_str(),
                card["type_line"].as_str(),
                card["oracle_text"].as_str(),
                card["mana_cost"].as_str(),
                card["cmc"].as_f64(),
                card["colors"].to_string(),
                card["rarity"].as_str(),
                card["image_uris"].to_string(),
            ])?;
        }
    }
    tx.commit()?;

    println!("Database created and populated successfully!");
    Ok(())
}