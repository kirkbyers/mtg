use rusqlite::{params, Result};
use serde_json::Value;
use std::{fs::File, io::Read, sync::{Arc, Mutex}, time::Duration};
use indicatif::{ProgressBar, ProgressStyle};
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Reading scryfall data");

    // Step 2: Read the JSON file
    let mut file_string = String::new();
    let spinner = ProgressBar::new_spinner();
    spinner.enable_steady_tick(Duration::from_millis(100));
    File::open("./data/scryfall-default-cards.json")?.read_to_string(&mut file_string)?;
    let cards_mutex = Arc::new(Mutex::new(serde_json::from_str::<Vec<Value>>(&file_string)?));
    spinner.finish();

    // Step 4: Create a progress bar
    let progress_bar = ProgressBar::new(cards_mutex.lock().unwrap().len() as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta})")?
            .progress_chars("#>-"),
    );
    let progress_bar = Arc::new(Mutex::new(progress_bar));
    

    println!("Processing cards");

    // Step 5: Split the cards into chunks for parallel processing
    let num_threads = 10;
    let chunk_size = (cards_mutex.lock().unwrap().len() + num_threads - 1) / num_threads;

    // Step 6: Create a vector to hold the handles to the spawned threads
    let mut handles = vec![];

    // Step 7: Spawn a thread for each chunk of cards
    for i in 0..num_threads {
        let cards = Arc::clone(&cards_mutex);
        let conn = mtg::db::init()?;
        let model = mtg::embedings::init()?;
        let progress_bar_clone = Arc::clone(&progress_bar);

        // Step 3: Insert data into the database
        let handle = thread::spawn(move || {
            let cards = cards.lock().unwrap();
            let start = i * chunk_size;
            let end = std::cmp::min(start + chunk_size, cards.len());
            let mut insert_card = conn.prepare(
                "INSERT OR REPLACE INTO cards (
                    source_id, name, set_code, collector_number, type_line, oracle_text,
                    mana_cost, cmc, colors, rarity, image_uris
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);",
            ).unwrap();
    
            let mut insert_card_vec = conn.prepare(
                "INSERT OR REPLACE INTO card_vecs (
                    rowid, embedding
                ) VALUES (?, ?);",
            ).unwrap();
            for card in &cards[start..end] {
                // Process each card and update the progress bar
                let card_vec = model.embed(vec![
                    format!("{:?} {:?}", card["name"].as_str(), card["oracle_text"].as_str())
                ], None).unwrap();
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
                ]).unwrap();
                insert_card_vec.execute(params![
                    &res_id,
                    &card_vec[0].iter().flat_map(|f| f.to_ne_bytes().to_vec()).collect::<Vec<_>>(),
                ]).unwrap();
                let pb = progress_bar_clone.lock().unwrap();
                pb.inc(1);
            }
        });

        handles.push(handle);
    }

    // Step 8: Wait for all threads to finish
    for handle in handles {
        handle.join().unwrap();
    }

    // Step 9: Finish and clear the progress bar
    progress_bar.lock().unwrap().finish();
    println!("Database created and populated successfully!");
    Ok(())
}