pub mod entry;

mod lemmatize;

use core::fmt;

pub use lemmatize::{lemmatize, lemmatize_from_file, remove_accents};

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
}

impl fmt::Display for WordClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            WordClass::Noun => write!(f, "noun"),
            WordClass::Adjective => write!(f, "adj"),
            WordClass::Adverb => write!(f, "adv"),
            WordClass::Verb => write!(f, "verb"),
            WordClass::Interjection => write!(f, "intj"),
            WordClass::Pronoun => write!(f, "pron"),
            WordClass::Preposition => write!(f, "prep"),
            WordClass::Determiner => write!(f, "det"),
            WordClass::Particle => write!(f, "particle"),
            WordClass::Conjunction => write!(f, "conj"),
        }
    }
}