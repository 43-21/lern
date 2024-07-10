use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

use tokio_rusqlite::{params, Connection, Result};


pub async fn create_table<P: AsRef<Path>>(path_to_frequencies: P) -> Result<()> {
    let conn = Connection::open("./db/database.db").await?;

    conn.call(
        |conn| {
            conn.execute(
                "CREATE TABLE frequency (
                    word_id INTEGER NOT NULL,
                    frequency INTEGER,
                    FOREIGN KEY(word_id) REFERENCES words(id)
                )",
                (),
            )?;

            conn.execute(
                "CREATE INDEX frequency_index ON frequency(word_id)",
                (),
            )?;
        
            Ok(())
        }
    ).await?;

    let start = std::time::Instant::now();
        
    insert_frequencies(&conn, path_to_frequencies).await?;

    let duration = start.elapsed();
    println!("Time elapsed for insertion: {:?}", duration);

    Ok(())
}

async fn insert_frequencies<P: AsRef<Path>>(conn: &Connection, file: P) -> Result<()> {
    let mut file = File::open(file).unwrap();
    let mut buffer= String::new();
    file.read_to_string(&mut buffer).unwrap();

    conn.call(
        move |conn| {
            let frequencies = buffer.split_whitespace();

            let mut select_stmt = conn.prepare("SELECT id FROM words WHERE word = ?1")?;
            let mut frequency_stmt = conn.prepare("INSERT INTO frequency VALUES (?1, ?2)")?;
        
            let mut word_ids = Vec::<(i64, usize)>::new();
        
            for (i, word) in frequencies.enumerate() {
                let word_iter = select_stmt.query_map(params![word], |row| {
                    row.get(0)
                })?;
        
        
                for word_id in word_iter {
                    let word_id: i64 = word_id.unwrap();
                    word_ids.push((word_id, i));
                }
            }
        
            for (word_id, i) in word_ids {
                frequency_stmt.execute(params![word_id, i as i64])?;
            }
        
            Ok(())
        }
    ).await?;

    Ok(())
}