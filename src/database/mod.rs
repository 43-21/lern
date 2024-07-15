use std::path::PathBuf;

use tokio::fs;
use tokio_rusqlite::{Connection, Result};

pub mod dictionary;
pub mod frequency;
pub mod schedule;
pub mod queue;

async fn init(conn: &Connection) -> Result<()> {
    conn.call(
        |conn| {
            conn.execute_batch(
                "PRAGMA journal_mode = WAL;
                --PRAGMA synchronous = OFF;
                PRAGMA journal_size_limit = 6144000;
                PRAGMA foreign_keys = ON;"
            )?;
                
            Ok(()) 
        }
    ).await
}

pub async fn create_schedule() -> Result<()> {
    fs::create_dir_all("./db").await.unwrap();

    let mut conn = Connection::open("./db/database.db").await?;

    init(&conn).await?;

    schedule::create_table(&mut conn).await?;

    Ok(())
}

pub async fn create_queue() -> Result<()> {
    fs::create_dir_all("./db").await.unwrap();

    let mut conn = Connection::open("./db/database.db").await?;

    init(&conn).await?;

    queue::create_table(&mut conn).await?;

    Ok(())
}

pub async fn create_dictionary(wiktionary_path: PathBuf) -> Result<()> {
    fs::create_dir_all("./db").await.unwrap();

    let conn = Connection::open("./db/database.db").await?;

    init(&conn).await?;
    
    dictionary::create_tables(wiktionary_path).await?;
    
    Ok(())
}

pub async fn create_frequency(frequency_path: PathBuf) -> Result<()> {
    fs::create_dir_all("./db").await.unwrap();

    let conn = Connection::open("./db/database.db").await?;

    init(&conn).await?;
    
    frequency::create_table(frequency_path).await?;
    
    Ok(())
}