use rusqlite::Connection;

pub fn init() -> Result<Connection, Box<dyn std::error::Error>>{
    let conn = Connection::open("./data/scryfall_cards.db")?;

    unsafe {
        conn.load_extension("./vec0.dylib", None)?
    };

    conn.execute(
        "CREATE TABLE IF NOT EXISTS cards (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_id TEXT,
            name TEXT,
            set_code TEXT,
            collector_number TEXT,
            type_line TEXT,
            oracle_text TEXT,
            mana_cost TEXT,
            cmc REAL,
            colors TEXT,
            rarity TEXT,
            image_uris TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_source_id ON cards (source_id);
        ",
        [],
    )?;


    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS card_vecs using vec0 (
            embedding float[384]
        )",
        [],
    )?;

    Ok(conn)
}