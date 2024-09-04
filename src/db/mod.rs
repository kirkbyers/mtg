pub mod vectors;

use anyhow::{anyhow, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use rusqlite::{ffi::sqlite3_auto_extension, named_params, params, Connection};
use serde::{Deserialize, Serialize};
use sqlite_vec::sqlite3_vec_init;
use tokio::sync::Mutex;
use vectors::{Point, SELECT_PAGINATED_SEMANTIC_SEARCH};

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

        CREATE INDEX IF NOT EXISTS idx_cards_set_code ON cards(set_code);
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

    conn.execute("
        CREATE TABLE IF NOT EXISTS card_cluster_assigments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            card_rowid INTEGER NOT NULL,
            cluster_id INTEGER NOT NULL,
            assigment_id INTEGER NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        CREATE INDEX IF NOT EXISTS idx_card_cluster_assigments_cluster_id ON card_cluster_assigments(cluster_id);
    ", [])?;

    Ok(conn)
}

pub fn prep_insert_card_cluster_assigments(
    conn: &Connection,
) -> rusqlite::Result<rusqlite::Statement> {
    conn.prepare(
        "INSERT OR REPLACE INTO card_cluster_assigments (
            card_rowid, cluster_id, assigment_id
        ) VALUES (?, ?, ?);",
    )
}

pub fn insert_cluster_assignments(
    conn: &Connection,
    assignments: &[usize],
    points: &[Point],
) -> Result<()> {
    let max_assignment_id: Option<i64> = conn.query_row(
        "SELECT MAX(assigment_id) FROM card_cluster_assigments;",
        [],
        |row| row.get(0),
    )?;

    let next_assignment_id = max_assignment_id.unwrap_or(0) + 1;
    let progress_bar = ProgressBar::new(assignments.len() as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} assignments (ETA: {eta})")
            .unwrap()
            .progress_chars("##-"),
    );
    let mut stmt = conn.prepare("INSERT INTO card_cluster_assigments (card_rowid, cluster_id, assigment_id) VALUES (?, ?, ?)")?;
    for (i, &cluster) in assignments.iter().enumerate() {
        let point = &points[i];
        stmt.execute(params![point.rowid, cluster as i64, next_assignment_id])?;
        progress_bar.inc(1);
    }
    progress_bar.finish_with_message("Assignments Saved");
    Ok(())
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

const SELECT_ALL_CARDS: &str = "
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
";

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

pub enum CardSearchType {
    Semantic,
    Like,
}

pub fn search_cards(
    conn: &Connection,
    search_query: &str,
    page: u32,
    page_size: u32,
    search_type: CardSearchType,
) -> Result<Vec<Card>> {
    let offset = (page - 1) * page_size;
    let limit = page_size;

    const PAGINATION_STMTS: &str = "
    GROUP BY c.name
    LIMIT :limit
    OFFSET :offset
    ";

    let stmt_str = if search_query.is_empty() {
        String::from(format!("{}{};", SELECT_ALL_CARDS, PAGINATION_STMTS))
    } else {
        match search_type {
            CardSearchType::Semantic => String::from(SELECT_PAGINATED_SEMANTIC_SEARCH),
            CardSearchType::Like => String::from(format!(
                "{} WHERE c.name LIKE :search COLLATE NOCASE {};",
                SELECT_ALL_CARDS, PAGINATION_STMTS
            )),
        }
    };

    let mut stmt = conn
        .prepare(&stmt_str)
        .context("Failed to prepare card search")?;
    let mut rows = if search_query.is_empty() {
        stmt.query(named_params! {":limit": &limit.to_string(), ":offset": &offset.to_string()})
            .context("Failed to execute prepared pagination no search")?
    } else {
        match search_type {
            CardSearchType::Semantic => {
                let embed_model = init().context("Failed to init fastembed")?;
                let embedded_search = string_to_embedding(&search_query, &embed_model)
                    .context("Failed to convert search to embedding")?;
                stmt.query(named_params! {
                    ":search": format!("{:?}", embedded_search),
                    ":limit": &limit.to_string(),
                    ":offset": &offset.to_string()
                })
                .context("Failed to execute prepared search")?
            }
            CardSearchType::Like => stmt
                .query(named_params! {
                    ":search": format!("%{}%", &search_query.to_string()),
                    ":limit": &limit.to_string(),
                    ":offset": &offset.to_string()
                })
                .context("Failed to execute prepared search")?,
        }
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
