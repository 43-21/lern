use tokio_rusqlite::{Connection, Result};

use crate::dictionary;

pub async fn create_table(conn: &mut Connection) -> Result<()> {
    conn.call(
        |conn| {
            conn.execute("DROP TABLE IF EXISTS lemmas", (),)?;
            conn.execute(
                "CREATE TABLE lemmas (
                    lemma TEXT PRIMARY KEY,
                    frequency INTEGER NOT NULL,
                    general_frequency INTEGER,
                    blacklisted INTEGER NOT NULL CHECK (blacklisted IN (0, 1))
                )",
                ()
            )?;        
            Ok(())
        }
    ).await?;

    Ok(())
}

pub async fn get_lemmas_queue(start: usize) -> Result<Vec<String>> {
    let conn = Connection::open("./db/database.db").await?;

    let queue = conn.call(move |conn| {
        let mut stmt = conn.prepare(
            "SELECT lemma FROM (
                SELECT lemma, frequency, general_frequency,
                ROW_NUMBER() OVER (
                    ORDER BY frequency DESC
                ) AS row_num_by_frequency,
                ROW_NUMBER() OVER (
                    ORDER BY general_frequency
                ) AS row_num_by_general_frequency
                FROM lemmas
                WHERE blacklisted = 0
            ) temp_table
            ORDER BY
                CASE
                    WHEN row_num_by_frequency < row_num_by_general_frequency THEN row_num_by_frequency
                    ELSE row_num_by_general_frequency
                END,
                CASE
                    WHEN row_num_by_frequency < row_num_by_general_frequency THEN row_num_by_general_frequency
                    ELSE row_num_by_frequency
                END
            LIMIT ?1,200")?;

        let rows = stmt.query_map([start], |row| {
            row.get::<usize, String>(0)
        })?;

        let mut queue = Vec::new();

        for row in rows {
            let lemma = row?;

            queue.push(lemma);
        }
        
        Ok(queue)
    }).await?;

    Ok(queue)
}

pub async fn blacklist_lemma(lemma: String) -> Result<()> {
    let conn = Connection::open("./db/database.db").await?;

    let lemma = dictionary::remove_accents(lemma);

    conn.call(
        move |conn| {
            conn.execute(
                "UPDATE lemmas
                SET blacklisted = 1
                WHERE lemma = ?1",
                [lemma]
            )?;


            Ok(())
        }
    ).await
}