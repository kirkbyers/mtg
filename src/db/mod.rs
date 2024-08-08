pub mod vectors;

use anyhow::{anyhow, Context, Result};
use rusqlite::{ffi::sqlite3_auto_extension, named_params, Connection};
use serde::{Deserialize, Serialize};
use sqlite_vec::sqlite3_vec_init;
use tokio::sync::Mutex;

use crate::embedings::{init, string_to_embedding};

// Wrapper for SQLite connection
pub struct DbConnection(pub Mutex<Connection>);

pub fn init_conn() -> Result<Connection> {
    unsafe {
        sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
    };
    println!("Mounted sqlite-vec");

    let conn = Connection::open("./data/scryfall_cards.db")?;

    let sqlite_vec_test: String = conn.query_row("SELECT vec_version();", [], |row| row.get(0))?;
    println!("{}", sqlite_vec_test);

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
) -> Result<(String, String, String, String, String, String)> {
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
        Err(anyhow!("No image URIs found".to_string()))
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
    pub image_url: Option<String>,
}

pub fn search_cards(
    conn: &Connection,
    query: &str,
    page: u32,
    page_size: u32,
) -> Result<Vec<Card>> {
    let offset = (page - 1) * page_size;
    let limit = page_size;

    let stmt_str = match query.is_empty() {
        true => String::from(
            "
            SELECT 
                c.id, c.oracle_id, c.name, c.lang, 
                c.released_at, c.mana_cost, c.cmc, 
                c.type_line, c.oracle_text, c.power, 
                c.toughness, c.rarity, c.flavor_text, 
                c.artist, c.set_code, c.collector_number, 
                c.digital, iu.normal
            FROM cards as c
            JOIN image_uris as iu
            ON c.id = iu.card_id 
            GROUP BY c.name
            ORDER BY c.name
            LIMIT :limit
            OFFSET :offset ;",
        ),
        false => String::from(
            "
            SELECT c.id, c.oracle_id, c.name, c.lang, 
                c.released_at, c.mana_cost, c.cmc, 
                c.type_line, c.oracle_text, c.power, 
                c.toughness, c.rarity, c.flavor_text, 
                c.artist, c.set_code, c.collector_number, 
                c.digital, iu.normal, cv.rowid, cv.distance
            FROM card_vecs as cv
            JOIN cards as c
            ON c.rowid = cv.rowid
            JOIN image_uris as iu
            ON c.id = iu.card_id
            WHERE embedding match :search
            and k = :limit
            GROUP BY c.name 
            ORDER BY distance
            LIMIT :limit
            OFFSET :offset ;",
        ),
    };

    let mut stmt = conn
        .prepare(&stmt_str)
        .context("Failed to prepare card search")?;
    let mut rows = if query.is_empty() {
        stmt.query(named_params! {":limit": &limit.to_string(), ":offset": &offset.to_string()})
            .context("Failed to execute prepared search")?
    } else {
        let embed_model = init().context("Failed to init fastembed")?;
        let embedded_search = string_to_embedding(&query, &embed_model)
            .context("Failed to convert search to embedding")?;
        stmt.query(named_params! {
            ":search": format!("{:?}", embedded_search),
            ":limit": &limit.to_string(),
            ":offset": &offset.to_string()
        })
        .context("Failed to execute prepared search")?
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
            image_url: row.get(17)?,
        };

        results.push(card);
    }

    Ok(results)
}
