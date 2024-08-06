use rusqlite::{Connection, Result, Statement};

pub fn get_vec_version_stmt(conn: &Connection) -> Result<Statement> {
    conn.prepare("SELECT vec_version();")
}
