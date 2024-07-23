use rusqlite::Connection;
use serde::{Serialize, Deserialize};

pub fn init_conn() -> Result<Connection, Box<dyn std::error::Error>> {
    let conn = Connection::open("./data/scryfall_cards.db")?;

    unsafe { conn.load_extension("./vec0.dylib", None)? };

    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS sets (
            code TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            set_type TEXT,
            released_at DATE
        );
        ",
        [],
    )?;
    conn.execute(
        "
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
    ",
        [],
    )?;

    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS card_vecs using vec0 (
            embedding float[384]
        )",
        [],
    )?;

    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS image_uris (
            card_id TEXT PRIMARY KEY,
            small TEXT,
            normal TEXT,
            large TEXT,
            png TEXT,
            art_crop TEXT,
            border_crop TEXT,
            FOREIGN KEY (card_id) REFERENCES cards(id)
        );
    ",
        [],
    )?;

    Ok(conn)
}

pub fn prep_insert_image_uris(conn: &Connection) -> rusqlite::Result<rusqlite::Statement> {
    conn.prepare(
        "INSERT OR REPLACE INTO image_uris (
            card_id, small, normal, large, png, art_crop, border_crop
        ) VALUES (?, ?, ?, ?, ?, ?, ?);",
    )
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
        "INSERT OR REPLACE INTO sets (code, name, set_type, released_at) VALUES (?, ?, ?, ?);",
    )
}

pub fn get_random_image_uris(
    conn: &Connection,
) -> Result<(String, String, String, String, String, String), Box<dyn std::error::Error>> {
    let mut stmt = conn.prepare("SELECT small, normal, large, png, art_crop, border_crop FROM image_uris ORDER BY random() LIMIT 1;")?;
    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        let small: String = row.get(0)?;
        let normal: String = row.get(1)?;
        let large: String = row.get(2)?;
        let png: String = row.get(3)?;
        let art_crop: String = row.get(4)?;
        let border_crop: String = row.get(5)?;
        Ok((small, normal, large, png, art_crop, border_crop))
    } else {
        Err("No image URIs found".into())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    pub oracle_id: String,
    pub name: String,
    pub lang: Option<String>,
    pub released_at: Option<String>,
    pub mana_cost: Option<String>,
    pub cmc: Option<f64>,
    pub type_line: Option<String>,
    pub oracle_text: Option<String>,
    pub power: Option<String>,
    pub toughness: Option<String>,
    pub rarity: Option<String>,
    pub flavor_text: Option<String>,
    pub artist: Option<String>,
    pub set_code: Option<String>,
    pub collector_number: Option<String>,
    pub digital: Option<String>,
}

pub fn search_cards(
    conn: &Connection,
    query: &str,
    page: u32,
    page_size: u32,
) -> Result<Vec<Card>, Box<dyn std::error::Error>> {
    let offset = (page - 1) * page_size;
    let limit = page_size;

    let mut stmt_str = String::from("SELECT id, oracle_id, name, lang, released_at, mana_cost, cmc, type_line, oracle_text, power, toughness, rarity, flavor_text, artist, set_code, collector_number, digital FROM cards ");

    if !query.is_empty() {
        stmt_str += "WHERE name LIKE ? ";
    }

    stmt_str += "ORDER BY name LIMIT ? OFFSET ?;";

    let mut stmt = conn.prepare(
        &stmt_str
    )?;
    let mut rows = if query.is_empty() {
        stmt.query(&[&limit.to_string(), &offset.to_string()])?
    } else {
        stmt.query(&[query, &limit.to_string(), &offset.to_string()])?
    };

    let mut results = Vec::new();
    while let Some(row) = rows.next()? {
        let card = Card {
            id: row.get(0)?,
            oracle_id: row.get(1)?,
            name: row.get(2)?,
            lang: row.get(3)?,
            released_at: row.get(4)?,
            mana_cost: row.get(5)?,
            cmc: row.get(6)?,
            type_line: row.get(7)?,
            oracle_text: row.get(8)?,
            power: row.get(9)?,
            toughness: row.get(10)?,
            rarity: row.get(11)?,
            flavor_text: row.get(12)?,
            artist: row.get(13)?,
            set_code: row.get(14)?,
            collector_number: row.get(15)?,
            digital: row.get(16)?,
        };

        results.push(card);
    }

    Ok(results)
}
