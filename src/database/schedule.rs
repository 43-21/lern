use tokio_rusqlite::{params, Connection, Result};

use crate::fsrs::card::Card;

pub async fn create_tables(conn: &mut Connection) -> Result<()> {
    conn.call(
        |conn| {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS cards (
                    id INTEGER PRIMARY KEY,
                    native TEXT NOT NULL,
                    russian TEXT NOT NULL,
                    due INTEGER NOT NULL,
                    stability REAL NOT NULL,
                    difficulty REAL NOT NULL
                )",
                (),
            )?;

            conn.execute(
                "CREATE TABLE IF NOT EXISTS lemmas (
                    lemma TEXT PRIMARY KEY,
                    frequency INTEGER NOT NULL,
                    general_frequency INTEGER NOT NULL,
                    blacklisted INTEGER NOT NULL CHECK (blacklisted IN (0, 1))
                )",
                ()
            )?;        
            Ok(())
        }
    ).await?;

    Ok(())
}

pub async fn get_due_cards(conn: &mut Connection, time: u64) -> Result<Vec<Card>> {
    let cards = conn.call(
        move |conn| {
            let ta = conn.transaction()?;
            let mut stmt = ta.prepare(
                "SELECT * FROM cards WHERE due > ?1"
            )?;
        
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
        }
    ).await?;

    Ok(cards)
}

pub async fn insert_card(card: Card) -> Result<()> {
    let conn = Connection::open("./db/database.db").await?;
    
    conn.call(
        move |conn| {
            conn.execute(
                "INSERT INTO cards(native, russian, due, stability, difficulty)
                VALUES (?1, ?2, ?3, ?4, ?5)",
                params![card.native, card.russian, card.due, card.stability, card.difficulty]
            )?;        
            Ok(())
        }
    ).await?;

    Ok(())
}

pub async fn update_cards(conn: &mut Connection, cards: Vec<Card>) -> Result<()> {
    conn.call(
        |conn| {
            let ta = conn.transaction()?;
            let mut stmt = ta.prepare(
                "UPDATE cards
                SET native = ?1,
                    russian = ?2,
                    due = ?3,
                    stability = ?4,
                    difficulty = ?5
                WHERE id = ?6"
            )?;
        
            for card in cards {
                stmt.execute(params![
                    card.native, card.russian, card.due, card.stability, card.difficulty, card.id
                ])?;
            }
        
            drop(stmt);
        
            ta.commit()?;
        
            Ok(())
        }
    ).await?;

    Ok(())
}

pub async fn get_lemmas_queue(start: usize) -> Result<Vec<String>> {
    let conn = Connection::open("./db/database.db").await?;

    let queue = conn.call(move |conn| {
        let mut stmt = conn.prepare("SELECT lemma FROM lemmas WHERE blacklisted = 0 ORDER BY id LIMIT ?1,200")?;

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

pub async fn insert_lemmas(lemmas: Vec<String>) -> Result<()> {
    let conn = Connection::open("./db/database.db").await?;
    conn.call(|conn| {
        let ta: rusqlite::Transaction = conn.transaction()?;
        let mut stmt = ta.prepare(
            "INSERT INTO lemmas
            VALUES (?1, 1, (SELECT frequency FROM frequency JOIN words ON words.id = frequency.word_id WHERE word = ?1), 0)
            ON CONFLICT(lemma) DO UPDATE SET frequency = frequency + 1"
        )?;

        for lemma in lemmas {
            stmt.execute([lemma])?;
        }

        drop(stmt);
        ta.commit()?;

        Ok(())
    }).await
}