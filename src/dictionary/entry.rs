#[derive(Debug, Clone)]
pub struct Entry {
    pub word: String,
    pub pos: String,
    pub etymology: Option<String>,
    pub senses: Vec<Sense>,
    pub forms: Vec<Form>,
    pub pronunciations: Vec<Pronunciation>,
}

#[derive(Debug, Clone)]
pub struct Form {
    pub form: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Sense {
    pub sense: String,
    pub examples: Vec<Example>,
    pub synonyms: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Pronunciation {
    pub ipa: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Example {
    pub text: String,
    pub english: Option<String>,
}