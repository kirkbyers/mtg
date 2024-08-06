use indicatif::{ProgressBar, ProgressStyle};
use mtg::db::{prep_insert_card, prep_insert_card_vec, prep_insert_image_uris, prep_insert_set};
use rusqlite::{params, Result, Row};
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
    let cards_count: u64 = cards.len() as u64;
    spinner.finish();

    let progress_bar = ProgressBar::new(cards_count);
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

        let _set_res_id = insert_set.insert(params![
            card["set"].as_str(),
            card["set_name"].as_str(),
            card["set_type"].as_str(),
            card["released_at"].as_str()
        ])?;
        let _res_id = insert_card.insert(params![
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

        progress_bar.inc(1);
    }
    progress_bar.finish();

    let progress_bar = ProgressBar::new(cards_count);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {msg} {pos}/{len} ({eta})")?
            .progress_chars("#>-"),
    );
    progress_bar.set_message("Processing embeddings");
    let page_size = 100;
    let mut get_card_info_page = conn.prepare(&format!(
        "SELECT 
            name, 
            COALESCE(oracle_text, ''),
            COALESCE(flavor_text, ''),
            COALESCE(power, ''),
            COALESCE(toughness, ''),
            COALESCE(type_line, ''),
            COALESCE(mana_cost, ''),
            rowid
        FROM cards c
        WHERE rowid >= ?
        LIMIT {};",
        page_size
    ))?;

    let card_info_mapper = |f: &Row| {
        let name: String = f.get(0)?;
        let oracle: String = f.get(1)?;
        let flavor: String = f.get(2)?;
        let power: String = f.get(3)?;
        let toughtness: String = f.get(4)?;
        let mana_cost: String = f.get(5)?;
        let type_line: String = f.get(6)?;

        Ok(format!("<name>{:?}<power>{:?}<toughness>{:?}<cost>{:?}<type>{:?}<oracle>{:?}<flavor>{:?}", &name, &power, &toughtness, &mana_cost, &type_line, &oracle, &flavor,))
    };

    // Loop through the newly stored data, and process the vector embeddings
    let mut offset = 0;
    loop {
        let card_info: Vec<String> = get_card_info_page
            .query_map(params![offset], card_info_mapper)?
            .map(|result| result.map_err(|e| e.into()))
            .collect::<Result<Vec<String>, Box<dyn std::error::Error>>>()?;

        if card_info.len() > 0 {
            let embeddings = model.embed(card_info.clone(), Some(page_size))?;
            for (idx, val) in embeddings.iter().enumerate() {
                insert_card_vec.execute(params![
                    idx + offset,
                    val.iter()
                        .flat_map(|f| f.to_ne_bytes().to_vec())
                        .collect::<Vec<_>>(),
                ])?;
            }
        }

        if card_info.len() < page_size {
            progress_bar.finish();
            break;
        }

        progress_bar.inc(card_info.len().try_into()?);
        offset += page_size
    }

    // Step 9: Finish and clear the progress bar
    progress_bar.finish();
    println!("Database created and populated successfully!");
    Ok(())
}
