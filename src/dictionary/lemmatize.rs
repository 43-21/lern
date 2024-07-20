use std::{collections::HashMap, path::PathBuf};

use regex::Regex;
use tokio::{fs::File, io::AsyncReadExt};
use tokio_rusqlite::Result;

use crate::database::dictionary;

pub async fn lemmatize(text: String) -> Result<()> {
    let regex = Regex::new(r"[^А-яёЁ]").unwrap();
    let forms: Vec<String> = regex.replace_all(&text, " ").split_whitespace().map(|s| s.to_lowercase()).collect();

    let mut hash_map = HashMap::new();
    for (i, form) in forms.into_iter().enumerate() {
        let entry = hash_map.entry(form).or_insert((0, i));
        entry.0 += 1;
    }

    dictionary::lemmatize(hash_map).await
}

pub async fn lemmatize_from_file(path: PathBuf) -> Result<()> {
    let mut file = File::open(path).await.unwrap();
    let mut text = String::new();
    file.read_to_string(&mut text).await.unwrap();


    let regex = Regex::new(r"[^А-яёЁ]").unwrap();
    let forms: Vec<String> = regex.replace_all(&text, " ").split_whitespace().map(|s| s.to_lowercase()).collect();

    let mut hash_map = HashMap::new();
    for (i, form) in forms.into_iter().enumerate() {
        let entry = hash_map.entry(form).or_insert((0, i));
        entry.0 += 1;
    }

    dictionary::lemmatize(hash_map).await
}

pub fn remove_accents(mut word: String) -> String {
    let patterns = vec![(r"а́", "а"), (r"е́", "е"), (r"и́", "и"), (r"о́", "о"), (r"у́", "у"), (r"э́", "э"), (r"ы́", "ы"), (r"ю́", "ю"), (r"я́", "я")];

    for (pattern, replacement) in patterns {
        let re = Regex::new(pattern).unwrap();
        word = re.replace_all(&word, replacement).to_string();
    }

    word
}
