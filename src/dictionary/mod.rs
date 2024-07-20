pub mod entry;

mod lemmatize;

pub use lemmatize::{lemmatize, lemmatize_from_file, remove_accents};
