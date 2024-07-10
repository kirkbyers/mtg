fn main() -> Result<(), Box<dyn std::error::Error>> {
    mtg::db::init_conn()?;

    Ok(())
}
