use std::{collections::{HashMap, HashSet}, path::PathBuf};

use tokio::fs;
use tokio_rusqlite::Connection;

use crate::Result;

pub mod dictionary;
pub mod frequency;
pub mod queue;
pub mod schedule;

async fn init(conn: &Connection) -> tokio_rusqlite::Result<()> {
    conn.call(|conn| {
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
                --PRAGMA synchronous = OFF;
                PRAGMA journal_size_limit = 6144000;
                PRAGMA foreign_keys = ON;",
        )?;

        Ok(())
    })
    .await
}

pub async fn create_schedule() -> Result<()> {
    fs::create_dir_all("./db").await?;

    let mut conn = Connection::open("./db/database.db").await?;

    init(&conn).await?;

    schedule::create_table(&mut conn).await?;

    Ok(())
}

pub async fn create_queue(keep_blacklist: bool) -> Result<()> {
    fs::create_dir_all("./db").await?;

    let mut conn = Connection::open("./db/database.db").await?;

    init(&conn).await?;

    queue::create_table(&mut conn, keep_blacklist).await?;

    Ok(())
}

pub async fn create_dictionary(wiktionary_path: PathBuf) -> Result<()> {
    fs::create_dir_all("./db").await?;

    let conn = Connection::open("./db/database.db").await?;

    init(&conn).await?;

    dictionary::create_tables(wiktionary_path).await?;

    Ok(())
}

pub async fn create_frequency(frequency_path: PathBuf) -> Result<()> {
    fs::create_dir_all("./db").await?;

    let conn = Connection::open("./db/database.db").await?;

    init(&conn).await?;

    frequency::create_table(frequency_path).await?;

    Ok(())
}

pub async fn check_tables() -> Result<HashMap<String, bool>> {
    let conn = Connection::open("./db/database.db").await?;

    let map = conn.call(|conn| {
        let mut map: HashMap<String, bool> = [
            (String::from("dictionary"), false),
            (String::from("frequency"), false),
            (String::from("cards"), false),
            (String::from("lemmas"), false),
        ].into_iter().collect();

        let mut dictionary_tables = HashSet::from([
            "words", "senses", "examples", "forms", "examples", "pronunciation", "synonyms", "sense_synonyms", "form_tags", "pronunciation_tags", "sense_tags",
        ]);

        let mut stmt = conn.prepare("SELECT name FROM sqlite_schema WHERE type = 'table'")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        for table in rows {
            let table: String = table?;
            dictionary_tables.remove(table.as_str());
            map.entry(table).and_modify(|e| *e = true);
        }

        if dictionary_tables.is_empty() {
            map.entry(String::from("dictionary")).and_modify(|e| *e = true);
        }

        Ok(map)
    }).await?;

    Ok(map)
}