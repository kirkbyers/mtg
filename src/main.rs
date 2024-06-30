use rusqlite::{Connection, LoadExtensionGuard};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("./data/scryfall_cards-ext.db")?;
    unsafe {
        let _guard = LoadExtensionGuard::new(&conn)?;
        conn.load_extension("sqlite-vec/dist/vec0.dylib", None)?;
    };

    Ok(())
}
