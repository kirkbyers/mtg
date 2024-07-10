use indicatif::{ProgressBar, ProgressStyle};
use mtg::db::{prep_insert_card, prep_insert_card_vec, prep_insert_image_uris, prep_insert_set};
use rusqlite::{params, Result};
use serde_json::Value;
use std::{fs::File, io::Read, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = mtg::db::init_conn()?;
    // Read the JSON file
    let mut file_string = String::new();
    let spinner = ProgressBar::new_spinner();
    spinner.set_message("Reading scryfall data");
    spinner.set_style(
        ProgressStyle::default_spinner().template("{spinner:.green} [{elapsed_precise}] {msg}")?,
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

    let mut insert_card = prep_insert_card(&conn)?;
    let mut insert_card_vec = prep_insert_card_vec(&conn)?;
    let mut insert_set = prep_insert_set(&conn)?;
    let mut insert_image_uris = prep_insert_image_uris(&conn)?;

    for card in cards {
        // Skipping non-english to save time processing
        if card["lang"].as_str() != Some("en") {
            progress_bar.inc(1);
            continue;
        }
        // Process each card and update the progress bar
        let card_vec = model.embed(
            vec![format!(
                "{:?} {:?} {:?}",
                card["name"].as_str(),
                card["oracle_text"].as_str(),
                card["flavor_text"].as_str(),
            )],
            None,
        )?;
        let _set_res_id = insert_set.insert(params![
            card["set"].as_str(),
            card["set_name"].as_str(),
            card["set_type"].as_str(),
            card["released_at"].as_str()
        ])?;
        let res_id = insert_card.insert(params![
            card["id"].as_str(),
            card["oracle_id"].as_str(),
            card["name"].as_str(),
            card["lang"].as_str(),
            card["released_at"].as_str(),
            card["mana_cost"].as_str(),
            card["cmc"].as_f64(),
            card["type_line"].as_str(),
            card["oracle_text"].as_str(),
            card["power"].as_str(),
            card["toughness"].as_str(),
            card["rarity"].as_str(),
            card["flavor_text"].as_str(),
            card["artist"].as_str(),
            card["set"].as_str(),
            card["collector_number"].to_string(),
            card["digital"].to_string(),
        ])?;
        if let Some(image_uris) = card["image_uris"].as_object() {
            let _image_res_id = insert_image_uris.insert(params![
                card["id"].as_str(),
                image_uris["small"].as_str(),
                image_uris["normal"].as_str(),
                image_uris["large"].as_str(),
                image_uris["png"].as_str(),
                image_uris["art_crop"].as_str(),
                image_uris["border_crop"].as_str(),
            ])?;
        }

        insert_card_vec.execute(params![
            &res_id,
            &card_vec[0]
                .iter()
                .flat_map(|f| f.to_ne_bytes().to_vec())
                .collect::<Vec<_>>(),
        ])?;
        progress_bar.inc(1);
    }

    // Step 9: Finish and clear the progress bar
    progress_bar.finish();
    println!("Database created and populated successfully!");
    Ok(())
}
