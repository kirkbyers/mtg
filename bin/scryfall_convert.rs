use rusqlite::{params, Result};
use serde_json::Value;
use std::{fs::File, io::Read, time::Duration};
use indicatif::{ProgressBar, ProgressStyle};

fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Read the JSON file
    let mut file_string = String::new();
    let spinner = ProgressBar::new_spinner();
    spinner.set_message("Reading scryfall data");
    spinner.set_style(
        ProgressStyle::default_spinner().template("{spinner:.green} [{elapsed_precise}] {msg}")?
    );
    spinner.enable_steady_tick(Duration::from_millis(100));
    File::open("./data/scryfall-default-cards.json")?.read_to_string(&mut file_string)?;
    let cards = serde_json::from_str::<Vec<Value>>(&file_string)?;
    spinner.finish();

    let progress_bar = ProgressBar::new(cards.len() as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {msg} {pos}/{len} ({eta})")?
            .progress_chars("#>-"),
    );
    progress_bar.set_message("Processing cards");
    let model = mtg::embedings::init()?;
    
    let conn = mtg::db::init()?;
    let mut insert_card = conn.prepare(
        "INSERT OR REPLACE INTO cards (
            source_id, name, set_code, collector_number, type_line, oracle_text,
            mana_cost, cmc, colors, rarity, image_uris
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);",
    )?;
    let mut insert_card_vec = conn.prepare(
        "INSERT OR REPLACE INTO card_vecs (
            rowid, embedding
        ) VALUES (?, ?);",
    )?;


    for card in cards {
        // Process each card and update the progress bar
        let card_vec = model.embed(vec![
            format!("{:?} {:?}", card["name"].as_str(), card["oracle_text"].as_str())
        ], None)?;
        let res_id = insert_card.insert(params![
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
        insert_card_vec.execute(params![
            &res_id,
            &card_vec[0].iter().flat_map(|f| f.to_ne_bytes().to_vec()).collect::<Vec<_>>(),
        ])?;
        progress_bar.inc(1);
    }


    // Step 9: Finish and clear the progress bar
    progress_bar.finish();
    println!("Database created and populated successfully!");
    Ok(())
}