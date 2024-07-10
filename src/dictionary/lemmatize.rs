use std::collections::HashMap;

use regex::Regex;
use tokio_rusqlite::Result;

use crate::database::dictionary;

pub async fn lemmatize(text: String) -> Result<HashMap<String, usize>> {
    let regex = Regex::new(r"[^А-яёЁ]").unwrap();
    let forms: Vec<String> = regex.replace_all(&text, " ")
        .split_whitespace()
        .map(|s| s.to_lowercase())
        .collect();

    let mut hash_map = HashMap::new();
    for form in forms {
        *hash_map.entry(form).or_insert(0) += 1;
    }

    dictionary::get_lemmas(hash_map).await
}

pub async fn lemmatize_2(text: String) -> Result<()> {
    let regex = Regex::new(r"[^А-яёЁ]").unwrap();
    let forms: Vec<String> = regex.replace_all(&text, " ")
        .split_whitespace()
        .map(|s| s.to_lowercase())
        .collect();

    let mut hash_map = HashMap::new();
    for form in forms {
        *hash_map.entry(form).or_insert(0) += 1;
    }

    dictionary::lemmatize(hash_map).await
}