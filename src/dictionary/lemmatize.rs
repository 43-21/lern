use std::{collections::HashMap, path::PathBuf};

use regex::Regex;
use tokio::{fs::File, io::AsyncReadExt};
use tokio_rusqlite::Result;

use crate::database::dictionary;

async fn lemmatize_sentences(text: String) -> Result<()> {
    let regex = Regex::new(r"[^\s»—][^\r\n\t\v\f.?!…]*[.?!…]+»*").unwrap();
    let sentences: Vec<String> = regex.find_iter(&text).map(|m| m.as_str().to_owned()).collect();

    let mut sentences_with_forms = Vec::new();

    let mut current_word_index = 0;
    for sentence in sentences {
        let regex = Regex::new(r"[^А-яёЁ]").unwrap();
        let forms: Vec<String> = regex.replace_all(&sentence, " ").split_whitespace().map(|s| s.to_lowercase()).collect();

        let mut tuples = Vec::new();
        for (i, form) in forms.into_iter().enumerate() {
            tuples.push((form, current_word_index + i))
        }
        current_word_index += tuples.len();

        sentences_with_forms.push((sentence, tuples));
    }

    dictionary::lemmatize_sentences(sentences_with_forms).await
}

pub async fn lemmatize(text: String, add_sentences: bool) -> Result<()> {
    if add_sentences {
        return lemmatize_sentences(text).await;
    }
    let regex = Regex::new(r"[^А-яёЁ]").unwrap();
    let forms: Vec<String> = regex.replace_all(&text, " ").split_whitespace().map(|s| s.to_lowercase()).collect();

    let mut hash_map = HashMap::new();
    for (i, form) in forms.into_iter().enumerate() {
        let entry = hash_map.entry(form).or_insert((0, i));
        entry.0 += 1;
    }

    dictionary::lemmatize(hash_map).await
}

pub async fn lemmatize_from_file(path: PathBuf, add_sentences: bool) -> Result<()> {
    let mut file = File::open(path).await.unwrap();
    let mut text = String::new();
    file.read_to_string(&mut text).await.unwrap();

    lemmatize(text, add_sentences).await
}

pub fn remove_accents(mut word: String) -> String {
    let patterns = vec![(r"а́", "а"), (r"е́", "е"), (r"и́", "и"), (r"о́", "о"), (r"у́", "у"), (r"э́", "э"), (r"ы́", "ы"), (r"ю́", "ю"), (r"я́", "я")];

    for (pattern, replacement) in patterns {
        let re = Regex::new(pattern).unwrap();
        word = re.replace_all(&word, replacement).to_string();
    }

    word
}
