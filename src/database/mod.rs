use tokio_rusqlite::{Connection, Result};

pub mod dictionary;
pub mod frequency;
pub mod schedule;

pub async fn create_schedule() -> Result<()> {
    let mut conn = Connection::open("./db/database.db").await?;

    conn.call(
        |conn| {
            conn.execute_batch(
                "PRAGMA journal_mode = WAL;
                PRAGMA synchronous = normal;
                PRAGMA journal_size_limit = 6144000;"
            )?;
                
            Ok(()) 
        }
    ).await?;

    schedule::create_tables(&mut conn).await?;

    Ok(())
}

pub async fn create_dictionary() -> Result<()> {
    let conn = Connection::open("./db/database.db").await?;
    
    conn.call(
        |conn| {
            conn.execute_batch(
                "PRAGMA journal_mode = WAL;
                PRAGMA synchronous = normal;
                PRAGMA journal_size_limit = 6144000;"
            )?;
                
            Ok(())
        }
    ).await?;

    dictionary::create_tables().await?;
    frequency::create_table("./frequency.txt").await?;
    
    Ok(())
}