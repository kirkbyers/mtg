use rusqlite::Connection;

pub fn init_conn() -> Result<Connection, Box<dyn std::error::Error>>{
    let conn = Connection::open("./data/scryfall_cards.db")?;

    unsafe {
        conn.load_extension("./vec0.dylib", None)?
    };

    conn.execute("
        CREATE TABLE IF NOT EXISTS sets (
            code TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            set_type TEXT,
            released_at DATE
        );
        ",
        [],
    )?;
    conn.execute("
        CREATE TABLE IF NOT EXISTS cards (
            id TEXT PRIMARY KEY,
            oracle_id TEXT,
            name TEXT NOT NULL,
            lang TEXT,
            released_at DATE,
            mana_cost TEXT,
            cmc REAL,
            type_line TEXT,
            oracle_text TEXT,
            power TEXT,
            toughness TEXT,
            rarity TEXT,
            flavor_text TEXT,
            artist TEXT,
            set_code TEXT,
            collector_number TEXT,
            digital BOOLEAN,
            FOREIGN KEY (set_code) REFERENCES sets(code)
        );
    ", [])?;

    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS card_vecs using vec0 (
            embedding float[384]
        )",
        [],
    )?;

    Ok(conn)
}

pub fn prep_insert_card(conn: &Connection) -> rusqlite::Result<rusqlite::Statement> {
    conn.prepare(
        "INSERT OR REPLACE INTO cards (
            id, oracle_id, name, lang, released_at, mana_cost, cmc,
            type_line, oracle_text, power, toughness, rarity, flavor_text, artist,
            set_code, collector_number, digital
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);",
    )
}

pub fn prep_insert_card_vec(conn: &Connection) -> rusqlite::Result<rusqlite::Statement> {
    conn.prepare(
        "INSERT OR REPLACE INTO card_vecs (
            rowid, embedding
        ) VALUES (?, ?);",
    )
}

pub fn prep_insert_set(conn: &Connection) -> rusqlite::Result<rusqlite::Statement> {
    conn.prepare(
        "INSERT OR REPLACE INTO sets (code, name, set_type, released_at) VALUES (?, ?, ?, ?);"
    )
}