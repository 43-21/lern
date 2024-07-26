use std::path::PathBuf;

use tokio::{fs, io::AsyncWriteExt};
use tokio_rusqlite::{params, Connection};

use crate::{fsrs::card::Card, Result};

use super::queue;

pub async fn create_table(conn: &mut Connection) -> Result<()> {
    conn.call(|conn| {
        conn.execute("DROP TABLE IF EXISTS cards", ())?;
        conn.execute(
            "CREATE TABLE cards (
                    id INTEGER PRIMARY KEY,
                    native TEXT NOT NULL,
                    russian TEXT NOT NULL,
                    due INTEGER NOT NULL,
                    stability REAL NOT NULL,
                    difficulty REAL NOT NULL
                )",
            (),
        )?;

        Ok(())
    })
    .await?;

    Ok(())
}

pub async fn get_due_cards(conn: &mut Connection, time: u64) -> Result<Vec<Card>> {
    let cards = conn
        .call(move |conn| {
            let ta = conn.transaction()?;
            let mut stmt = ta.prepare("SELECT * FROM cards WHERE due > ?1")?;

            let cards_iter = stmt.query_map([time], |row| {
                Ok(Card {
                    id: row.get(0)?,
                    native: row.get(1)?,
                    russian: row.get(2)?,
                    due: row.get(3)?,
                    stability: row.get(4)?,
                    difficulty: row.get(5)?,
                })
            })?;
            let mut cards = Vec::new();

            for card in cards_iter {
                cards.push(card?);
            }

            drop(stmt);
            ta.commit()?;

            Ok(cards)
        })
        .await?;

    Ok(cards)
}

pub async fn insert_card(card: Card) -> Result<()> {
    let conn = Connection::open("./db/database.db").await?;

    queue::blacklist_lemma(card.russian.clone()).await?;

    conn.call(move |conn| {
        conn.execute(
            "INSERT INTO cards(native, russian, due, stability, difficulty)
                VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                card.native,
                card.russian,
                card.due,
                card.stability,
                card.difficulty
            ],
        )?;
        Ok(())
    })
    .await?;

    Ok(())
}

pub async fn update_cards(conn: &mut Connection, cards: Vec<Card>) -> Result<()> {
    conn.call(|conn| {
        let ta = conn.transaction()?;
        let mut stmt = ta.prepare(
            "UPDATE cards
                SET native = ?1,
                    russian = ?2,
                    due = ?3,
                    stability = ?4,
                    difficulty = ?5
                WHERE id = ?6",
        )?;

        for card in cards {
            stmt.execute(params![
                card.native,
                card.russian,
                card.due,
                card.stability,
                card.difficulty,
                card.id
            ])?;
        }

        drop(stmt);

        ta.commit()?;

        Ok(())
    })
    .await?;

    Ok(())
}

pub async fn export(path: PathBuf) -> Result<()> {
    let mut file = fs::File::create(path)
        .await
        .expect("error when attempting to create file");

    let conn = Connection::open("./db/database.db").await?;
    let buffer = conn
        .call(move |conn| {
            let mut buffer = String::new();

            buffer += "#seperator:Semicolon\n";
            buffer += "#html:false\n";

            let mut stmt = conn.prepare("SELECT russian, native FROM cards")?;

            let rows = stmt.query_map((), |row| Ok((row.get(0)?, row.get(1)?)))?;

            for row in rows {
                let (russian, native): (String, String) = row?;
                buffer += format!("{russian};{native}\n").as_str();
            }

            Ok(buffer)
        })
        .await?;

    file.write_all(buffer.as_bytes())
        .await
        .expect("error when writing into file");

    Ok(())
}
