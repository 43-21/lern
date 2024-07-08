use regex::Regex;
use tokio_rusqlite::Result;

use crate::database::dictionary;

pub async fn lemmatize(text: String) -> Result<Vec<String>> {
    let regex = Regex::new(r"[^А-яёЁ]").unwrap();
    let words: Vec<String> = regex.replace_all(&text, " ")
        .split_whitespace()
        .map(|s| s.to_lowercase())
        .collect();

    dictionary::get_lemmas(words).await
}