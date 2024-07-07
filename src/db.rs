use rusqlite::Connection;

pub fn init() -> Result<Connection, Box<dyn std::error::Error>>{
    let conn = Connection::open("./data/scryfall_cards.db")?;

    unsafe {
        sqlite_vec::sqlite3_vec_init();
    };

    conn.execute(
        "CREATE TABLE IF NOT EXISTS cards (
            id TEXT PRIMARY KEY,
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
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS card_vecs using vec0 (
            card_id TEXT PRIMARY KEY,
            embedding float[]
        )",
        [],
    )?;

    Ok(conn)
}