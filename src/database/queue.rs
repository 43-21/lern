use tokio_rusqlite::{Connection, Result};

use crate::dictionary;

pub async fn create_table(conn: &mut Connection, keep_blacklist: bool) -> Result<()> {
    conn.call(move |conn| {
        let row: rusqlite::Result<String> = conn.query_row("SELECT name FROM sqlite_schema WHERE type='table' AND name='lemmas'", [], |row| row.get(0));

        let table_exists = row.is_ok();

        if table_exists && keep_blacklist {
            conn.execute("DELETE FROM lemmas WHERE blacklisted = 0", ())?;
        } else {
            conn.execute_batch(
                "DROP TABLE IF EXISTS sentences;
                DROP INDEX IF EXISTS sentence_index;
                CREATE TABLE sentences (
                    lemma TEXT NOT NULL,
                    sentence TEXT NOT NULL,
                    FOREIGN KEY(lemma) REFERENCES lemmas(lemma) ON DELETE CASCADE
                );
                CREATE INDEX sentence_index ON sentences(lemma)",
            )?;

            conn.execute_batch(
                "DROP TABLE IF EXISTS lemmas;
                CREATE TABLE lemmas (
                    lemma TEXT PRIMARY KEY,
                    frequency INTEGER NOT NULL,
                    general_frequency INTEGER,
                    blacklisted INTEGER NOT NULL CHECK (blacklisted IN (0, 1)),
                    first_occurence INTEGER NOT NULL
                );",
            )?;
        }

        Ok(())
    })
    .await?;

    Ok(())
}

pub async fn get_lemmas_queue(start: usize, by_frequency: bool, by_general_frequency: bool, by_first_occurence: bool) -> Result<Vec<String>> {
    let conn = Connection::open("./db/database.db").await?;

    let queue = conn
        .call(move |conn| {
            let mut stmt = if by_frequency && by_general_frequency && by_first_occurence {
                conn.prepare(
                    "SELECT lemma,
            MIN(row_num_by_frequency, row_num_by_general_frequency, 1.5 * row_num_by_first_occurence) AS smallest,
            MAX(row_num_by_frequency, row_num_by_general_frequency, 1.5 * row_num_by_first_occurence) AS largest
            FROM (
                SELECT lemma,
                ROW_NUMBER() OVER (
                    ORDER BY frequency DESC
                ) AS row_num_by_frequency,
                ROW_NUMBER() OVER (
                    ORDER BY general_frequency
                ) AS row_num_by_general_frequency,
                ROW_NUMBER() OVER (
                    ORDER BY first_occurence
                ) AS row_num_by_first_occurence
                FROM lemmas
                WHERE blacklisted = 0
            ) temp_table
            ORDER BY
                smallest,
                CASE
                    WHEN smallest < row_num_by_frequency AND row_num_by_frequency < largest
                        THEN row_num_by_frequency
                    WHEN smallest < row_num_by_general_frequency AND row_num_by_general_frequency < largest
                        THEN row_num_by_general_frequency
                    ELSE 1.5 * row_num_by_first_occurence
                END,
                largest
            LIMIT ?1,200",
                )?
            } else if by_frequency && by_general_frequency {
                conn.prepare(
                    "SELECT lemma,
            MIN(row_num_by_frequency, row_num_by_general_frequency) AS smallest,
            MAX(row_num_by_frequency, row_num_by_general_frequency) AS largest
            FROM (
                SELECT lemma,
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
                smallest,
                largest
            LIMIT ?1,200",
                )?
            } else if by_frequency && by_first_occurence {
                conn.prepare(
                    "SELECT lemma,
            MIN(row_num_by_frequency, 1.5 * row_num_by_first_occurence) AS smallest,
            MAX(row_num_by_frequency, 1.5 * row_num_by_first_occurence) AS largest
            FROM (
                SELECT lemma,
                ROW_NUMBER() OVER (
                    ORDER BY frequency DESC
                ) AS row_num_by_frequency,
                ROW_NUMBER() OVER (
                    ORDER BY first_occurence
                ) AS row_num_by_first_occurence
                FROM lemmas
                WHERE blacklisted = 0
            ) temp_table
            ORDER BY
                smallest,
                largest
            LIMIT ?1,200",
                )?
            } else if by_general_frequency && by_first_occurence {
                conn.prepare(
                    "SELECT lemma,
            MIN(row_num_by_general_frequency, 1.5 * row_num_by_first_occurence) AS smallest,
            MAX(row_num_by_general_frequency, 1.5 * row_num_by_first_occurence) AS largest
            FROM (
                SELECT lemma,
                ROW_NUMBER() OVER (
                    ORDER BY general_frequency
                ) AS row_num_by_general_frequency,
                ROW_NUMBER() OVER (
                    ORDER BY first_occurence
                ) AS row_num_by_first_occurence
                FROM lemmas
                WHERE blacklisted = 0
            ) temp_table
            ORDER BY
                smallest,
                largest
            LIMIT ?1,200",
                )?
            } else if by_frequency {
                conn.prepare(
                    "SELECT lemma
                FROM lemmas
                WHERE blacklisted = 0
            ORDER BY frequency
            LIMIT ?1,200",
                )?
            } else if by_general_frequency {
                conn.prepare(
                    "SELECT lemma
                FROM lemmas
                WHERE blacklisted = 0
            ORDER BY general_frequency
            LIMIT ?1,200",
                )?
            } else if by_first_occurence {
                conn.prepare(
                    "SELECT lemma
                FROM lemmas
                WHERE blacklisted = 0
            ORDER BY first_occurence
            LIMIT ?1,200",
                )?
            } else {
                conn.prepare(
                    "SELECT lemma
                FROM lemmas
                WHERE blacklisted = 0
                LIMIT ?1,200",
                )?
            };

            let rows = stmt.query_map([start], |row| row.get::<usize, String>(0))?;

            let mut queue = Vec::new();

            for row in rows {
                let lemma = row?;

                queue.push(lemma);
            }

            Ok(queue)
        })
        .await?;

    Ok(queue)
}

pub async fn blacklist_lemma(lemma: String) -> Result<()> {
    let conn = Connection::open("./db/database.db").await?;

    let lemma = dictionary::remove_accents(lemma);

    conn.call(move |conn| {
        conn.execute(
            "UPDATE lemmas
                SET blacklisted = 1
                WHERE lemma = ?1",
            [lemma],
        )?;

        Ok(())
    })
    .await
}

pub async fn get_sentences(lemma: String) -> Result<Vec<String>> {
    let conn = Connection::open("./db/database.db").await?;

    conn.call(move |conn| {
        let mut stmt = conn.prepare("SELECT sentence FROM sentences WHERE lemma = ?1")?;

        let rows = stmt.query_map([lemma], |row| row.get(0))?;

        let mut sentences = Vec::new();

        for sentence in rows {
            sentences.push(sentence?);
        }

        Ok(sentences)
    })
    .await
}
