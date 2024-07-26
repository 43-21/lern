use std::collections::HashSet;

use tokio_rusqlite::Connection;

use crate::{dictionary::{self, WordClass}, Result};

pub async fn create_table(conn: &mut Connection, keep_blacklist: bool) -> Result<()> {
    conn.call(move |conn| {
        let row: rusqlite::Result<String> = conn.query_row(
            "SELECT name FROM sqlite_schema WHERE type='table' AND name='lemmas'",
            [],
            |row| row.get(0),
        );

        let table_exists = row.is_ok();

        if table_exists && keep_blacklist {
            conn.execute("DELETE FROM lemmas WHERE blacklisted = 0", ())?;
        } else {
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

        Ok(())
    })
    .await?;

    Ok(())
}

pub async fn get_lemmas_queue(
    start: usize,
    by_frequency: bool,
    by_general_frequency: bool,
    by_first_occurence: bool,
    word_classes: HashSet<WordClass>,
) -> Result<Vec<String>> {
    let conn = Connection::open("./db/database.db").await?;

    let queue = conn
        .call(move |conn| {
            let mut args = Vec::new();
            let mut row_number_clauses = Vec::new();

            if by_frequency {
                args.push("row_num_by_frequency");
                row_number_clauses.push(
                    "ROW_NUMBER() OVER (
                        ORDER BY frequency DESC
                    ) AS row_num_by_frequency",
                );
            }
            if by_general_frequency {
                args.push("row_num_by_general_frequency");
                row_number_clauses.push(
                    "ROW_NUMBER() OVER (
                        ORDER BY general_frequency
                    ) AS row_num_by_general_frequency",
                );
            }
            if by_first_occurence {
                args.push("1.5 * row_num_by_first_occurence");
                row_number_clauses.push(
                    "ROW_NUMBER() OVER (
                        ORDER BY first_occurence
                    ) AS row_num_by_first_occurence",
                );
            }

            let (join_str, where_str) = if word_classes.is_empty() {
                ("", String::from(""))
            }
            else {
                let mut string = String::from("AND pos IN (");
                for class in word_classes {
                    string += format!("'{}', ", class).as_str();
                }
                string = string.trim_end_matches(", ").to_owned() + ")";
                ("JOIN words ON lemma = word", string)
            };

            let (min_max_string, order_by_string) = {
                if row_number_clauses.len() == 1 {
                    if by_frequency {
                        (String::from(",\nrow_num_by_frequency"), String::from("ORDER BY row_num_by_frequency"))
                    }
                    else if by_general_frequency {
                        (String::from(",\nrow_num_by_general_frequency"), String::from("ORDER BY row_num_by_general_frequency"))
                    }
                    else {
                        (String::from(",\nrow_num_by_first_occurence"), String::from("ORDER BY row_num_by_first_occurence"))
                    }
                }
                else {
                    let args_string = args.join(", ");
                    (
                        format!(
                            ",\nMIN({}) AS smallest,
                            MAX({}) AS largest",
                            args_string,
                            args_string,
                        ),
                        format!(
                            "ORDER BY
                            smallest{},
                            largest",
                            if by_frequency && by_general_frequency && by_first_occurence {
                                String::from(
                                    ",\nCASE
                                        WHEN smallest < row_num_by_frequency AND row_num_by_frequency < largest
                                            THEN row_num_by_frequency
                                        WHEN smallest < row_num_by_general_frequency AND row_num_by_general_frequency < largest
                                            THEN row_num_by_general_frequency
                                        ELSE 1.5 * row_num_by_first_occurence
                                    END",
                                )
                            } else {
                                String::from("")
                            }
                        )
                    )
                }
            };

            let query = if !args.is_empty() {
                format!(
                    "SELECT lemma{}
                    FROM (
                        SELECT lemma,
                        {}
                        FROM lemmas {}
                        WHERE blacklisted = 0 {} GROUP BY lemma
                    ) temp_table
                    {}
                    LIMIT ?1,200",
                    min_max_string,
                    row_number_clauses.join(",\n"),
                    join_str,
                    where_str,
                    order_by_string,
                )
            } else {
                String::from("SELECT lemma FROM lemmas WHERE blacklisted = 0 LIMIT ?1,200")
            };

            let mut stmt = conn.prepare(&query)?;

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

pub async fn blacklist_lemma(lemma: String) -> tokio_rusqlite::Result<()> {
    let conn = Connection::open("./db/database.db").await?;

    let lemma = dictionary::remove_accents(lemma)?;

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

pub async fn get_sentences(lemma: String) -> tokio_rusqlite::Result<Vec<String>> {
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
