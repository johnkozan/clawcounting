use refinery::embed_migrations;
use rusqlite::Connection;

embed_migrations!("migrations");

pub fn run_migrations(conn: &mut Connection) -> Result<(), Box<dyn std::error::Error>> {
    migrations::runner().run(conn)?;
    Ok(())
}
