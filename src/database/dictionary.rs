use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use tokio_rusqlite::{params, Connection, Transaction};

use crate::dictionary::entry::{Entry, Example, Form, Pronunciation, Sense};
use crate::dictionary::{self, WordClass};
use crate::error::Error;
use crate::Result;

fn read_lines<P>(filename: P) -> Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub async fn create_tables(path_to_wiktionary: PathBuf) -> Result<()> {
    let conn = Connection::open("./db/database.db").await?;

    conn.call(|conn| {
        conn.execute_batch(
            "DROP TABLE IF EXISTS form_tags;
            DROP TABLE IF EXISTS pronunciation_tags;                
            DROP TABLE IF EXISTS sense_tags;
            DROP TABLE IF EXISTS sense_synonyms;
            DROP TABLE IF EXISTS examples;
            DROP TABLE IF EXISTS pronunciation;
            DROP TABLE IF EXISTS senses;
            DROP TABLE IF EXISTS synonyms;
            DROP TABLE IF EXISTS forms;
            DROP TABLE IF EXISTS frequency;
            DROP TABLE IF EXISTS words;",
        )?;

        conn.execute_batch(
            "DROP INDEX IF EXISTS word_index;
            DROP INDEX IF EXISTS word_form_index;
            DROP INDEX IF EXISTS form_tag_index;
            DROP INDEX IF EXISTS sense_index;
            DROP INDEX IF EXISTS example_index;
            DROP INDEX IF EXISTS sense_synonym_index;
            DROP INDEX IF EXISTS pronunciation_index;
            DROP INDEX IF EXISTS normalized_form_index;",
        )?;

        conn.execute(
            "CREATE TABLE words (
                id INTEGER PRIMARY KEY,
                word TEXT NOT NULL,
                pos TEXT NOT NULL,
                etymology TEXT,
                expansion TEXT
            )",
            (),
        )?;

        conn.execute(
            "CREATE TABLE senses (
                id INTEGER PRIMARY KEY,
                word_id INTEGER NOT NULL,
                sense TEXT,
                relevance INTEGER NOT NULL,
                FOREIGN KEY(word_id) REFERENCES words(id)
            )",
            (),
        )?;

        conn.execute(
            "CREATE TABLE examples (
                sense_id INTEGER NOT NULL,
                text TEXT NOT NULL,
                english TEXT,
                FOREIGN KEY(sense_id) REFERENCES senses(id)
            )",
            (),
        )?;

        conn.execute(
            "CREATE TABLE pronunciation (
                id INTEGER PRIMARY KEY,
                word_id INTEGER NOT NULL,
                ipa TEXT NOT NULL,
                FOREIGN KEY(word_id) REFERENCES words(id)
            )",
            (),
        )?;

        conn.execute(
            "CREATE TABLE forms (
                id INTEGER PRIMARY KEY,
                word_id INTEGER NOT NULL,
                form TEXT NOT NULL,
                normalized_form TEXT NOT NULL,
                FOREIGN KEY(word_id) REFERENCES words(id)
            )",
            (),
        )?;

        conn.execute(
            "CREATE TABLE synonyms (
                id INTEGER PRIMARY KEY,
                synonym TEXT NOT NULL
            )",
            (),
        )?;

        conn.execute(
            "CREATE TABLE sense_synonyms (
                sense_id INTEGER NOT NULL,
                synonym_id INTEGER NOT NULL,
                FOREIGN KEY(sense_id) REFERENCES senses(id),
                FOREIGN KEY(synonym_id) REFERENCES synonyms(id)
            )",
            (),
        )?;

        conn.execute(
            "CREATE TABLE form_tags (
                form_id INTEGER NOT NULL,
                tag TEXT NOT NULL,
                FOREIGN KEY(form_id) REFERENCES forms(id)
            )",
            (),
        )?;

        conn.execute(
            "CREATE TABLE pronunciation_tags (
                pronunciation_id INTEGER NOT NULL,
                tag TEXT NOT NULL,
                FOREIGN KEY(pronunciation_id) REFERENCES pronunciation(id)
            )",
            (),
        )?;

        conn.execute(
            "CREATE TABLE sense_tags (
                sense_id INTEGER NOT NULL,
                tag TEXT NOT NULL,
                FOREIGN KEY(sense_id) REFERENCES senses(id)
            )",
            (),
        )?;

        conn.execute_batch(
            "CREATE INDEX word_index ON words(word);
            CREATE INDEX word_form_index ON forms(word_id);
            CREATE INDEX form_tag_index ON form_tags(form_id);
            CREATE INDEX sense_index ON senses(word_id);
            CREATE INDEX example_index ON examples(sense_id);
            CREATE INDEX sense_synonym_index ON sense_synonyms(sense_id);
            CREATE INDEX pronunciation_index ON pronunciation(word_id);
            CREATE INDEX normalized_form_index ON forms(normalized_form);",
        )?;

        let start = std::time::Instant::now();

        let mut ta = conn.transaction()?;
        insert_data(&mut ta, path_to_wiktionary).map_err(Into::<tokio_rusqlite::Error>::into)?;
        ta.commit()?;

        let duration = start.elapsed();
        println!("Time elapsed for insertion: {:?}", duration);

        Ok(())
    })
    .await?;

    conn.close().await?;
    Ok(())
}

fn insert_data(ta: &mut Transaction, path_to_wiktionary: PathBuf) -> Result<()> {
    let mut word_stmt_expansion =
        ta.prepare("INSERT INTO words (word, pos, expansion) VALUES (?1, ?2, ?3)")?;
    let mut word_stmt_etymology_expansion =
        ta.prepare("INSERT INTO words (word, pos, etymology, expansion) VALUES (?1, ?2, ?3, ?4)")?;
    let mut word_stmt_etymology =
        ta.prepare("INSERT INTO words (word, pos, etymology) VALUES (?1, ?2, ?3)")?;
    let mut word_stmt = ta.prepare("INSERT INTO words (word, pos) VALUES (?1, ?2)")?;

    let mut sense_stmt_gloss =
        ta.prepare("INSERT INTO senses (word_id, sense, relevance) VALUES (?1, ?2, ?3)")?;
    let mut sense_stmt = ta.prepare("INSERT INTO senses (word_id, relevance) VALUES (?1, ?2)")?;
    let mut sense_tag_stmt =
        ta.prepare("INSERT INTO sense_tags (sense_id, tag) VALUES (?1, ?2)")?;

    let mut example_stmt_english =
        ta.prepare("INSERT INTO examples (sense_id, text, english) VALUES (?1, ?2, ?3)")?;
    let mut example_stmt = ta.prepare("INSERT INTO examples (sense_id, text) VALUES (?1, ?2)")?;

    let mut synonym_stmt = ta.prepare("INSERT INTO synonyms (synonym) VALUES (?1)")?;
    let mut sense_synonym_stmt =
        ta.prepare("INSERT INTO sense_synonyms (sense_id, synonym_id) VALUES (?1, ?2)")?;

    let mut form_stmt =
        ta.prepare("INSERT INTO forms (form, word_id, normalized_form) VALUES (?1, ?2, ?3)")?;
    let mut form_tag_stmt = ta.prepare("INSERT INTO form_tags (form_id, tag) VALUES (?1, ?2)")?;

    let mut pronunciation_stmt =
        ta.prepare("INSERT INTO pronunciation (word_id, ipa) VALUES (?1, ?2)")?;
    let mut pronunciation_tag_stmt =
        ta.prepare("INSERT INTO pronunciation_tags (pronunciation_id, tag) VALUES (?1, ?2)")?;

    let lines = read_lines(path_to_wiktionary)?;

    'iteration: for (i, line) in lines.map_while(std::io::Result::ok).enumerate() {
        let json: serde_json::Value = serde_json::from_str(&line)?;

        let word = json
            .get("word")
            .ok_or(Error::GetValueFailed(json.clone(), i))?;
        let word = word
            .as_str()
            .ok_or(Error::ValueConversionFailed(word.to_owned(), i))?;

        let pos_value = json
            .get("pos")
            .ok_or(Error::GetValueFailed(json.clone(), i))?;
        let pos_str = pos_value
            .as_str()
            .ok_or(Error::ValueConversionFailed(pos_value.to_owned(), i))?;
        let pos = WordClass::from(pos_str);

        let etymology = match json.get("etymology_text") {
            Some(value) => Some(
                value
                    .as_str()
                    .ok_or(Error::ValueConversionFailed(value.to_owned(), i))?,
            ),
            None => None,
        };

        if pos == WordClass::Unknown {
            continue 'iteration;
        }

        let head_templates = json.get("head_templates");
        let expansion = {
            if let Some(head_templates) = head_templates {
                let head_templates = head_templates
                    .as_array()
                    .ok_or(Error::ValueConversionFailed(head_templates.to_owned(), i))?
                    .first()
                    .ok_or(Error::EmptyJSONArray(i))?;

                head_templates
                    .get("expansion")
                    .ok_or(Error::GetValueFailed(head_templates.to_owned(), i))?
                    .as_str()
            } else {
                None
            }
        };

        let json_senses = json
            .get("senses")
            .ok_or(Error::GetValueFailed(json.clone(), i))?;
        let json_senses = json_senses
            .as_array()
            .ok_or(Error::ValueConversionFailed(json_senses.to_owned(), i))?;
        let mut senses = Vec::new();

        'senses: for sense in json_senses {
            if sense.get("form_of").is_some() {
                continue;
            }

            if let Some(tags) = sense.get("tags") {
                for tag in tags
                    .as_array()
                    .ok_or(Error::ValueConversionFailed(tags.to_owned(), i))?
                {
                    if tag
                        .as_str()
                        .ok_or(Error::ValueConversionFailed(tag.to_owned(), i))?
                        == "form-of"
                    {
                        continue 'senses;
                    }
                }
            }

            senses.push(sense);
        }

        if senses.is_empty() {
            continue 'iteration;
        }

        if let (Some(etymology), Some(expansion)) = (etymology, expansion) {
            word_stmt_etymology_expansion.execute([
                word,
                pos.to_string().as_str(),
                etymology,
                expansion,
            ])?;
        } else if let Some(etymology) = etymology {
            word_stmt_etymology.execute([word, pos.to_string().as_str(), etymology])?;
        } else if let Some(expansion) = expansion {
            word_stmt_expansion.execute([word, pos.to_string().as_str(), expansion])?;
        } else {
            word_stmt.execute([word, pos.to_string().as_str()])?;
        }

        let word_id = ta.last_insert_rowid();

        let senses = json
            .get("senses")
            .ok_or(Error::GetValueFailed(json.clone(), i))?;
        let senses = senses
            .as_array()
            .ok_or(Error::ValueConversionFailed(senses.to_owned(), i))?;
        for (i, sense) in senses.iter().enumerate() {
            let glosses = sense.get("glosses");
            let gloss = if let Some(glosses) = glosses {
                let glosses = glosses
                    .as_array()
                    .ok_or(Error::ValueConversionFailed(glosses.to_owned(), i))?;
                let gloss = glosses.first().ok_or(Error::EmptyJSONArray(i))?;
                Some(
                    gloss
                        .as_str()
                        .ok_or(Error::ValueConversionFailed(gloss.to_owned(), i))?,
                )
            } else {
                None
            };

            if let Some(gloss) = gloss {
                sense_stmt_gloss.execute(params![word_id, gloss, i])?;
            } else {
                sense_stmt.execute(params![word_id, i])?;
            }

            let sense_id = ta.last_insert_rowid();

            let tags = {
                if let Some(tags) = sense.get("tags") {
                    tags.as_array()
                        .ok_or(Error::ValueConversionFailed(tags.to_owned(), i))?
                        .to_owned()
                } else {
                    Vec::<Value>::new()
                }
            };

            for tag in tags {
                let tag = tag
                    .as_str()
                    .ok_or(Error::ValueConversionFailed(tag.to_owned(), i))?;

                sense_tag_stmt.execute(params![sense_id, tag])?;
            }

            let examples = {
                if let Some(examples) = sense.get("examples") {
                    examples
                        .as_array()
                        .ok_or(Error::ValueConversionFailed(examples.to_owned(), i))?
                        .to_owned()
                } else {
                    Vec::<Value>::new()
                }
            };

            for example in examples {
                let text = example
                    .get("text")
                    .ok_or(Error::GetValueFailed(example.to_owned(), i))?;
                let text = text
                    .as_str()
                    .ok_or(Error::ValueConversionFailed(text.to_owned(), i))?;

                let english = example.get("english");
                let english = match english {
                    Some(english) => Some(
                        english
                            .as_str()
                            .ok_or(Error::ValueConversionFailed(english.to_owned(), i))?,
                    ),
                    None => None,
                };

                if let Some(english) = english {
                    example_stmt_english.execute(params![sense_id, text, english])?;
                } else {
                    example_stmt.execute(params![sense_id, text])?;
                }
            }

            if let Some(synonyms) = sense.get("synonyms") {
                let synonyms = synonyms
                    .as_array()
                    .ok_or(Error::ValueConversionFailed(synonyms.to_owned(), i))?;

                for synonym in synonyms {
                    let synonym = synonym
                        .as_str()
                        .ok_or(Error::ValueConversionFailed(synonym.to_owned(), i))?;

                    synonym_stmt.execute([synonym])?;
                    let synonym_id = ta.last_insert_rowid();
                    sense_synonym_stmt.execute([sense_id, synonym_id])?;
                }
            }
        }

        if let Some(forms) = json.get("forms") {
            let forms = forms
                .as_array()
                .ok_or(Error::ValueConversionFailed(forms.to_owned(), i))?;

            'forms: for form in forms {
                let word = form
                    .get("form")
                    .ok_or(Error::GetValueFailed(form.to_owned(), i))?;
                let word = word
                    .as_str()
                    .ok_or(Error::ValueConversionFailed(word.to_owned(), i))?;

                let source = form.get("source");
                if source.is_none() {
                    continue 'forms;
                }
                let source = source.unwrap();
                let source = source
                    .as_str()
                    .ok_or(Error::ValueConversionFailed(source.to_owned(), i))?;
                if source != "declension" && source != "conjugation" {
                    continue 'forms;
                }

                let tags = form.get("tags");
                if tags.is_none() {
                    continue 'forms;
                }
                let tags = tags.unwrap();
                let tags = tags
                    .as_array()
                    .ok_or(Error::ValueConversionFailed(tags.to_owned(), i))?;
                for tag in tags {
                    match tag
                        .as_str()
                        .ok_or(Error::ValueConversionFailed(tag.to_owned(), i))?
                    {
                        "inflection-template" | "table-tags" | "class" => continue 'forms,
                        _ => (),
                    }
                }

                let normalized = dictionary::remove_accents(word.to_owned())?;

                form_stmt.execute(params![word, word_id, &normalized])?;
                let form_id = ta.last_insert_rowid();

                for tag in tags {
                    let tag = tag
                        .as_str()
                        .ok_or(Error::ValueConversionFailed(tag.to_owned(), i))?;
                    form_tag_stmt.execute(params![form_id, tag])?;
                }
            }
        }

        if let Some(sounds) = json.get("sounds") {
            let sounds = sounds
                .as_array()
                .ok_or(Error::ValueConversionFailed(sounds.to_owned(), i))?;
            for sound in sounds {
                if let Some(ipa) = sound.get("ipa") {
                    let ipa = ipa
                        .as_str()
                        .ok_or(Error::ValueConversionFailed(ipa.to_owned(), i))?;
                    pronunciation_stmt.execute(params![word_id, ipa])?;
                    let pronunciation_id = ta.last_insert_rowid();

                    if let Some(tags) = sound.get("tags") {
                        let tags = tags
                            .as_array()
                            .ok_or(Error::ValueConversionFailed(tags.to_owned(), i))?;
                        for tag in tags {
                            let tag = tag
                                .as_str()
                                .ok_or(Error::ValueConversionFailed(tag.to_owned(), i))?;
                            pronunciation_tag_stmt.execute(params![pronunciation_id, tag])?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

pub async fn read_entries(word: String) -> Result<Vec<Entry>> {
    let conn = Connection::open("./db/database.db").await?;

    let entries = conn
        .call(|conn| {
            let ta = conn.transaction()?;
            let mut word_stmt = ta.prepare("SELECT * FROM words WHERE word = ?1")?;

            let mut form_stmt = ta.prepare(
                "SELECT forms.id, form FROM forms
                WHERE word_id = ?1",
            )?;

            let mut form_tag_stmt = ta.prepare("SELECT tag FROM form_tags WHERE form_id = ?1")?;

            let mut sense_stmt =
                ta.prepare("SELECT id, sense FROM senses WHERE word_id = ?1 ORDER BY relevance")?;

            let mut example_stmt =
                ta.prepare("SELECT text, english FROM examples WHERE sense_id = ?1")?;

            let mut synonym_stmt = ta.prepare(
                "SELECT synonym FROM synonyms
                JOIN sense_synonyms ON synonyms.id = synonym_id
                JOIN senses ON senses.id = sense_id
                WHERE sense_id = ?1",
            )?;

            let mut sense_tag_stmt = ta
                .prepare("SELECT tag FROM sense_tags JOIN senses ON sense_id = id WHERE id = ?1")?;

            let mut pronunciation_stmt =
                ta.prepare("SELECT id, ipa FROM pronunciation WHERE word_id = ?1")?;

            let mut pronunciation_tag_stmt = ta.prepare(
                "SELECT tag FROM pronunciation_tags
                JOIN pronunciation ON id = pronunciation_id
                WHERE id = ?1",
            )?;

            let mut entries = Vec::new();

            let entry_iter = word_stmt.query_map([word], |row| {
                let id: i64 = row.get(0)?;
                let word: String = row.get(1)?;
                let pos: WordClass = row.get(2)?;
                let etymology: Option<String> = row.get(3)?;
                let expansion: Option<String> = row.get(4)?;
                Ok((id, word, pos, etymology, expansion))
            })?;

            for entry in entry_iter {
                let (id, word, pos, etymology, expansion) = entry?;

                let mut forms = Vec::new();
                let mut senses = Vec::new();
                let mut pronunciations = Vec::new();

                let form_iter = form_stmt.query_map([id], |row| {
                    let id: i64 = row.get(0)?;
                    let form: String = row.get(1)?;

                    Ok((id, form))
                })?;

                for form in form_iter {
                    let (id, form) = form?;

                    let mut tags = Vec::new();

                    let tag_iter =
                        form_tag_stmt.query_map([id], |row| row.get::<usize, String>(0))?;

                    for tag in tag_iter {
                        tags.push(tag?);
                    }

                    forms.push(Form { form, tags });
                }

                let sense_iter = sense_stmt.query_map([id], |row| {
                    let id: i64 = row.get(0)?;
                    let sense: String = row.get(1)?;

                    Ok((id, sense))
                })?;

                for sense in sense_iter {
                    let (id, sense) = sense?;

                    let mut examples = Vec::new();
                    let mut synonyms = Vec::new();
                    let mut tags = Vec::new();

                    let example_iter = example_stmt.query_map([id], |row| {
                        let text: String = row.get(0)?;
                        let english: Option<String> = row.get(1)?;

                        Ok((text, english))
                    })?;

                    for example in example_iter {
                        let (text, english) = example?;
                        examples.push(Example { text, english })
                    }

                    let synonym_iter =
                        synonym_stmt.query_map([id], |row| row.get::<usize, String>(0))?;

                    for synonym in synonym_iter {
                        synonyms.push(synonym?);
                    }

                    let tag_iter =
                        sense_tag_stmt.query_map([id], |row| row.get::<usize, String>(0))?;

                    for tag in tag_iter {
                        tags.push(tag?);
                    }

                    senses.push(Sense {
                        sense,
                        examples,
                        synonyms,
                        tags,
                    })
                }

                let pronunciation_iter = pronunciation_stmt.query_map([id], |row| {
                    let id: i64 = row.get(0)?;
                    let ipa: String = row.get(1)?;

                    Ok((id, ipa))
                })?;

                for pronunciation in pronunciation_iter {
                    let (id, ipa) = pronunciation?;

                    let mut tags = Vec::new();

                    let tag_iter = pronunciation_tag_stmt
                        .query_map([id], |row| row.get::<usize, String>(0))?;

                    for tag in tag_iter {
                        tags.push(tag?);
                    }

                    pronunciations.push(Pronunciation { ipa, tags })
                }

                entries.push(Entry {
                    word,
                    pos,
                    etymology,
                    expansion,
                    senses,
                    forms,
                    pronunciations,
                })
            }

            drop(word_stmt);
            drop(form_stmt);
            drop(form_tag_stmt);
            drop(sense_stmt);
            drop(sense_tag_stmt);
            drop(example_stmt);
            drop(synonym_stmt);
            drop(pronunciation_stmt);
            drop(pronunciation_tag_stmt);

            ta.commit()?;

            Ok(entries)
        })
        .await?;

    Ok(entries)
}

pub async fn lemmatize_sentences(sentences: Vec<(String, Vec<(String, usize)>)>) -> Result<()> {
    let conn = Connection::open("./db/database.db").await?;
    conn.call(|conn| {
        let ta = conn.transaction()?;

        let max_first_occurence: usize = ta.query_row("SELECT MAX(first_occurence) FROM lemmas", [], |row| row.get(0)).unwrap_or(0);

        let mut insert_lemmas_stmt = ta.prepare(
            "INSERT INTO lemmas
                SELECT w.word as lemma, 1 as frequency, frequency.frequency as general_frequency, 0 as blacklisted, ?1 as first_occurence
                FROM words w
                JOIN forms ON forms.word_id = w.id
                JOIN frequency ON w.id = frequency.word_id
                WHERE normalized_form = ?2
                GROUP BY w.id
            ON CONFLICT(lemma) DO UPDATE SET frequency = frequency + 1",
        )?;

        let mut insert_sentence_stmt = ta.prepare(
            "INSERT INTO sentences
            SELECT lemma, ?1
            FROM (
                SELECT word as lemma
                FROM words
                JOIN forms ON forms.word_id = words.id
                JOIN frequency ON words.id = frequency.word_id
                WHERE normalized_form = ?2
                GROUP BY words.word
            ) AS lemmas WHERE (SELECT COUNT(*) FROM sentences WHERE lemma = lemmas.lemma) < 5
                AND NOT EXISTS (SELECT 1 FROM sentences WHERE lemma = lemmas.lemma AND sentence = ?1)",
        )?;

        let start = std::time::Instant::now();
        for (sentence, forms) in sentences {
            let size = forms.len();
            for (form, position) in forms {
                insert_lemmas_stmt.execute(params![max_first_occurence + position, form])?;
                if (3..20).contains(&size) {
                    insert_sentence_stmt.execute(params![sentence, form])?;
                }
            }
        }

        drop(insert_lemmas_stmt);
        drop(insert_sentence_stmt);

        ta.commit()?;

        println!("elapsed: {:?}", start.elapsed());

        Ok(())
    })
    .await?;

    Ok(())
}

pub async fn lemmatize(forms: HashMap<String, (usize, usize)>) -> Result<()> {
    let conn = Connection::open("./db/database.db").await?;
    conn.call(|conn| {
        let ta = conn.transaction()?;

        let max_first_occurence: usize = ta.query_row("SELECT MAX(first_occurence) FROM lemmas", [], |row| row.get(0)).unwrap_or(0);

        let mut stmt: rusqlite::Statement = ta.prepare(
            "INSERT INTO lemmas
            SELECT w.word, ?1 as frequency, frequency.frequency as general_frequency, 0 as blacklisted, ?2 as first_occurence
                FROM words w
                JOIN forms ON forms.word_id = w.id
                JOIN frequency ON w.id = frequency.word_id
                WHERE normalized_form = ?3
                GROUP BY w.id
            ON CONFLICT(lemma) DO UPDATE SET frequency = frequency + ?1",
        )?;

        let start = std::time::Instant::now();
        for (form, (frequency, first_occurence)) in forms {
            stmt.execute(params![frequency, max_first_occurence + first_occurence, form])?;
        }
        drop(stmt);

        ta.commit()?;

        println!("elapsed: {:?}", start.elapsed());

        Ok(())
    })
    .await?;

    Ok(())
}
