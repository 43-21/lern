pub mod entry;

mod lemmatize;

use core::fmt;

pub use lemmatize::{lemmatize, lemmatize_from_file, remove_accents};
use rusqlite::types::FromSql;

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum WordClass {
    Noun,
    Verb,
    Adjective,
    Adverb,
    Determiner,
    Particle,
    Interjection,
    Conjunction,
    Pronoun,
    Preposition,
    Unknown,
}

impl fmt::Display for WordClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            WordClass::Noun => write!(f, "noun"),
            WordClass::Adjective => write!(f, "adjective"),
            WordClass::Adverb => write!(f, "adverb"),
            WordClass::Verb => write!(f, "verb"),
            WordClass::Interjection => write!(f, "interjection"),
            WordClass::Pronoun => write!(f, "pronoun"),
            WordClass::Preposition => write!(f, "preposition"),
            WordClass::Determiner => write!(f, "determiner"),
            WordClass::Particle => write!(f, "particle"),
            WordClass::Conjunction => write!(f, "conjunction"),
            WordClass::Unknown => write!(f, "unknown"),
        }
    }
}

impl From<&str> for WordClass {
    fn from(value: &str) -> Self {
        match value {
            "noun" => Self::Noun,
            "verb" => Self::Verb,
            "adj" => Self::Adjective,
            "adjective" => Self::Adjective,
            "adv" => Self::Adverb,
            "adverb" => Self::Adverb,
            "intj" => Self::Interjection,
            "interjection" => Self::Interjection,
            "pron" => Self::Pronoun,
            "pronoun" => Self::Pronoun,
            "prep" => Self::Preposition,
            "preposition" => Self::Preposition,
            "det" => Self::Determiner,
            "determiner" => Self::Determiner,
            "particle" => Self::Particle,
            "conj" => Self::Conjunction,
            "conjunction" => Self::Conjunction,
            _ => Self::Unknown,
        }
    }
}

impl FromSql for WordClass {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        value.as_str().map(WordClass::from)
    }
}