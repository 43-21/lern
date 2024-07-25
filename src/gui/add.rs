use std::collections::HashSet;

use iced::{
    alignment::{Horizontal, Vertical},
    border::Radius,
    widget::{
        markdown,
        text_input::{focus, Id},
        Button, Checkbox, Column, Container, Row, Scrollable, Text, TextInput,
    },
    Alignment, Border, Element, Length, Task, Theme,
};
use iced_aw::{
    menu::{DrawPath, Item, Menu},
    menu_bar, menu_items,
    style::{menu_bar, Status},
    TabLabel,
};
use once_cell::sync::Lazy;

use crate::{
    database::{dictionary, queue, schedule},
    dictionary::{entry::Entry, WordClass},
    fsrs::card::Card,
};

use super::Tab;

static INPUT_ID: Lazy<Id> = Lazy::new(Id::unique);

#[derive(Debug, Clone)]
pub enum Message {
    RussianChanged(String),
    NativeChanged(String),
    DictionaryTimer {
        version: usize,
    },
    ReadEntries {
        preloading: bool,
        word: String,
    },
    EntriesRead {
        preloading: bool,
        entries: Vec<Entry>,
    },
    ReadSentences {
        preloading: bool,
        word: String,
    },
    SentencesRead {
        preloading: bool,
        sentences: Vec<String>,
    },
    LoadNext,
    Preload,
    Add,
    FromQueue(bool),
    ReadFromQueue,
    QueueRead {
        lemmas: Vec<String>,
    },
    Blacklist,
    Ignore,
    OrderButtonPressed,
    OrderFrequency(bool),
    OrderGeneralFrequency(bool),
    OrderFirstOccurence(bool),
    ClassButtonPressed,
    ClassToggled(bool, WordClass),
    LinkClicked(markdown::Url),
    Error(String),
}

pub struct AddTab {
    russian: String,
    native: String,
    version: usize,
    entries: Vec<Entry>,
    sentences: Vec<String>,
    lemmas: Vec<String>,
    from_queue: bool,
    order_frequency: bool,
    order_general_frequency: bool,
    order_first_occurence: bool,
    word_classes: HashSet<WordClass>,
    ignored_from_queue: usize,
    next_word: Option<(String, Vec<Entry>)>,
    next_sentences: Option<Vec<String>>,
    markdown_items: Option<Vec<markdown::Item>>,
}

impl AddTab {
    pub fn new() -> AddTab {
        AddTab {
            russian: String::new(),
            native: String::new(),
            version: 0,
            entries: Vec::new(),
            sentences: Vec::new(),
            lemmas: Vec::new(),
            from_queue: false,
            ignored_from_queue: 0,
            next_word: None,
            next_sentences: None,
            order_frequency: true,
            order_general_frequency: true,
            order_first_occurence: true,
            word_classes: HashSet::from([
                WordClass::Noun,
                WordClass::Adjective,
                WordClass::Adverb,
                WordClass::Conjunction,
                WordClass::Determiner,
                WordClass::Interjection,
                WordClass::Particle,
                WordClass::Preposition,
                WordClass::Pronoun,
                WordClass::Verb,
            ]),
            markdown_items: None,
        }
    }

    fn set_entry_markdown_items(&mut self) {
        let mut entry_string = String::new();
    
        for entry in &self.entries {
            let etymology: Option<&String> = entry.etymology.as_ref();
            let word = {
                if let Some(expansion) = &entry.expansion {
                    expansion
                } else {
                    &entry.word
                }
            };
    
            if !entry.pronunciations.is_empty() {
                entry_string += "__Pronunciation__\n\n";
            }
    
            for pronunciation in &entry.pronunciations {
                let tag_string = {
                    if pronunciation.tags.is_empty() {
                        String::new()
                    } else {
                        format!(" (*{}*)", pronunciation.tags.join(", "))
                    }
                };
    
                entry_string += &format!("* {}{}\n\n", pronunciation.ipa, tag_string);
            }

            entry_string += &format!("__{}__\n\n{}\n\n", entry.pos, word);
            if let Some(etymology) = etymology {
                entry_string += &format!("{}\n\n", etymology);
            }
    
            for (i, sense) in entry.senses.iter().enumerate() {
                let tag_string = {
                    if sense.tags.is_empty() {
                        String::new()
                    } else {
                        format!(" (*{}*)", sense.tags.join(", "))
                    }
                };
    
                entry_string += &format!("{}. {}{}\n\n", i + 1, sense.sense, tag_string);
    
                for example in &sense.examples {
                    let translation = if let Some(example) = &example.english {
                        format!(" - {}", example)
                    } else {
                        String::new()
                    };
    
                    entry_string += &format!("\t{}{}\n\n", example.text, translation);
                }
            }
        }
    
        for sentence in &self.sentences {
            entry_string += format!("{}\n\n", sentence).as_str();
        }
    
        self.markdown_items = Some(markdown::parse(&entry_string, Theme::TokyoNight.palette()).collect());
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::RussianChanged(value) => {
                self.russian = value;
                self.version += 1;
                let version = self.version;
                Task::perform(
                    async move {
                        tokio::time::sleep(tokio::time::Duration::from_millis(75)).await;
                        version
                    },
                    |version| Message::DictionaryTimer { version },
                )
            }
            Message::NativeChanged(value) => {
                self.native = value;
                Task::none()
            }
            Message::Add => {
                if !self.native.is_empty() && !self.russian.is_empty() {
                    Task::perform(
                        schedule::insert_card(Card::new(&self.native, &self.russian)),
                        |_| Message::LoadNext,
                    )
                } else {
                    Task::none()
                }
            }
            Message::DictionaryTimer { version } => {
                if version == self.version {
                    Task::batch(vec![
                        Task::done(Message::ReadEntries {
                            preloading: false,
                            word: self.russian.clone(),
                        }),
                        Task::done(Message::ReadSentences {
                            preloading: false,
                            word: self.russian.clone(),
                        }),
                    ])
                } else {
                    Task::none()
                }
            }
            Message::EntriesRead {
                preloading,
                entries,
            } => {
                if preloading {
                    let expansion = {
                        if !entries.is_empty() {
                            entries.first().unwrap().expansion.clone()
                        } else {
                            None
                        }
                    };
                    let with_accent = if let Some(expansion) = expansion {
                        expansion.split_whitespace().next().unwrap_or("").to_owned()
                    } else {
                        String::new()
                    };
                    self.next_word = Some((with_accent, entries));
                } else {
                    self.entries = entries;
                    self.set_entry_markdown_items()
                }

                if self.russian.is_empty() && preloading {
                    Task::done(Message::LoadNext)
                } else {
                    Task::none()
                }
            }
            Message::ReadFromQueue => {
                if self.from_queue {
                    Task::future(queue::get_lemmas_queue(
                        self.ignored_from_queue,
                        self.order_frequency,
                        self.order_general_frequency,
                        self.order_first_occurence,
                        self.word_classes.clone(),
                    ))
                    .then(move |lemmas| match lemmas {
                        Ok(lemmas) => Task::done(Message::QueueRead { lemmas }),
                        Err(e) => Task::done(Message::Error(e.to_string())),
                    })
                } else {
                    Task::none()
                }
            }
            Message::QueueRead { lemmas } => {
                self.lemmas = lemmas;
                if self.lemmas.is_empty() {
                    self.from_queue = false;
                }
                if self.next_word.is_none() {
                    Task::done(Message::Preload)
                } else {
                    Task::none()
                }
            }
            Message::Blacklist => {
                Task::perform(queue::blacklist_lemma(self.russian.clone()), |_| {
                    Message::LoadNext
                })
            }
            Message::Ignore => {
                self.ignored_from_queue += 1;
                Task::done(Message::LoadNext)
            }
            Message::Error(message) => {
                println!("{message}");
                Task::none()
            }
            Message::ReadEntries { preloading, word } => Task::perform(
                dictionary::read_entries(word),
                move |entries| match entries {
                    Ok(entries) => Message::EntriesRead {
                        preloading,
                        entries,
                    },
                    Err(e) => Message::Error(e.to_string()),
                },
            ),
            Message::ReadSentences { preloading, word } => Task::perform(
                queue::get_sentences(word),
                move |sentences| match sentences {
                    Ok(sentences) => Message::SentencesRead {
                        preloading,
                        sentences,
                    },
                    Err(e) => Message::Error(e.to_string()),
                },
            ),
            Message::SentencesRead {
                preloading,
                sentences,
            } => {
                if preloading {
                    self.next_sentences = Some(sentences);
                } else {
                    self.sentences = sentences;
                    self.set_entry_markdown_items();
                }

                Task::none()
            }
            Message::FromQueue(value) => {
                self.from_queue = value;
                Task::done(Message::LoadNext)
            }
            Message::LoadNext => {
                self.version = 0;
                self.native = String::new();
                self.russian = String::new();

                if !self.from_queue {
                    return Task::none();
                }

                if let Some((lemma, entries)) = &self.next_word {
                    lemma.clone_into(&mut self.russian);
                    entries.clone_into(&mut self.entries);
                    self.set_entry_markdown_items();
                    self.next_word = None;
                }

                if let Some(sentences) = &self.next_sentences {
                    sentences.clone_into(&mut self.sentences);
                    self.set_entry_markdown_items();
                    self.next_sentences = None;
                }

                Task::batch([Task::done(Message::Preload), focus(INPUT_ID.clone())])
            }
            Message::Preload => {
                if self.lemmas.is_empty() {
                    Task::done(Message::ReadFromQueue)
                } else {
                    let word = self.lemmas.remove(0);
                    Task::batch(vec![
                        Task::done(Message::ReadEntries {
                            preloading: true,
                            word: word.clone(),
                        }),
                        Task::done(Message::ReadSentences {
                            preloading: true,
                            word: word.clone(),
                        }),
                    ])
                }
            }
            Message::OrderButtonPressed => Task::none(),
            Message::OrderFrequency(value) => {
                self.order_frequency = value;
                Task::none()
            }
            Message::OrderGeneralFrequency(value) => {
                self.order_general_frequency = value;
                Task::none()
            }
            Message::OrderFirstOccurence(value) => {
                self.order_first_occurence = value;
                Task::none()
            }
            Message::ClassButtonPressed => Task::none(),
            Message::ClassToggled(value, class) => {
                if value {
                    self.word_classes.insert(class);
                } else {
                    self.word_classes.remove(&class);
                }
                Task::none()
            }
            Message::LinkClicked(_link) => {
                Task::none()
            }
        }
    }
}

impl Tab for AddTab {
    type Message = super::Message;

    fn title(&self) -> String {
        String::from("Add")
    }

    fn tab_label(&self) -> TabLabel {
        TabLabel::Text(self.title())
    }

    fn content(&self) -> iced::Element<'_, Self::Message> {
        let entry_markdown = if let Some(markdown_items) = &self.markdown_items {
            Some(markdown(markdown_items, markdown::Settings::default())
            .map(Message::LinkClicked))
        } else { None };

        let entry_scrollable = match entry_markdown {
            None => None,
            Some(entry_markdown) => Some(Scrollable::new(entry_markdown).width(Length::Fill))
        };

        let word_class_menu = menu_bar!((
            Button::new(Text::new("Include...")).on_press(Message::ClassButtonPressed),
            {
                Menu::new(menu_items!((Checkbox::new(
                    "nouns",
                    self.word_classes.contains(&WordClass::Noun)
                )
                .on_toggle(|value| Message::ClassToggled(value, WordClass::Noun))
                .width(Length::Fill))(
                    Checkbox::new("verbs", self.word_classes.contains(&WordClass::Verb))
                        .on_toggle(|value| Message::ClassToggled(value, WordClass::Verb))
                        .width(Length::Fill)
                )(
                    Checkbox::new(
                        "adjectives",
                        self.word_classes.contains(&WordClass::Adjective)
                    )
                    .on_toggle(|value| Message::ClassToggled(value, WordClass::Adjective))
                    .width(Length::Fill)
                )(
                    Checkbox::new(
                        "determiners",
                        self.word_classes.contains(&WordClass::Determiner)
                    )
                    .on_toggle(|value| Message::ClassToggled(value, WordClass::Determiner))
                    .width(Length::Fill)
                )(
                    Checkbox::new("adverbs", self.word_classes.contains(&WordClass::Adverb))
                        .on_toggle(|value| Message::ClassToggled(value, WordClass::Adverb))
                        .width(Length::Fill)
                )(
                    Checkbox::new(
                        "interjections",
                        self.word_classes.contains(&WordClass::Interjection)
                    )
                    .on_toggle(|value| Message::ClassToggled(value, WordClass::Interjection))
                    .width(Length::Fill)
                )(
                    Checkbox::new(
                        "particles",
                        self.word_classes.contains(&WordClass::Particle)
                    )
                    .on_toggle(|value| Message::ClassToggled(value, WordClass::Particle))
                    .width(Length::Fill)
                )(
                    Checkbox::new(
                        "conjunctions",
                        self.word_classes.contains(&WordClass::Conjunction)
                    )
                    .on_toggle(|value| Message::ClassToggled(value, WordClass::Conjunction))
                    .width(Length::Fill)
                )(
                    Checkbox::new(
                        "prepositions",
                        self.word_classes.contains(&WordClass::Preposition)
                    )
                    .on_toggle(|value| Message::ClassToggled(value, WordClass::Preposition))
                    .width(Length::Fill)
                )(
                    Checkbox::new("pronouns", self.word_classes.contains(&WordClass::Pronoun))
                        .on_toggle(|value| Message::ClassToggled(value, WordClass::Pronoun))
                        .width(Length::Fill)
                )))
                .max_width(180.0)
                .offset(15.0)
                .spacing(5.0)
            }
        ))
        .draw_path(DrawPath::Backdrop)
        .style(|theme: &iced::Theme, status: Status| iced_aw::menu::Style {
            path_border: Border {
                radius: Radius::new(6.0),
                ..Default::default()
            },
            ..menu_bar::primary(theme, status)
        });

        let order_menu = menu_bar!((
            Button::new(Text::new("Order by...")).on_press(Message::OrderButtonPressed),
            {
                Menu::new(menu_items!((Checkbox::new(
                    "frequency in texts",
                    self.order_frequency
                )
                .on_toggle(Message::OrderFrequency)
                .width(Length::Fill))(
                    Checkbox::new("general frequency", self.order_general_frequency)
                        .on_toggle(Message::OrderGeneralFrequency)
                        .width(Length::Fill)
                )(
                    Checkbox::new("first occurence", self.order_first_occurence)
                        .on_toggle(Message::OrderFirstOccurence)
                        .width(Length::Fill)
                )))
                .max_width(180.0)
                .offset(15.0)
                .spacing(5.0)
            }
        ))
        .draw_path(DrawPath::Backdrop)
        .style(|theme: &iced::Theme, status: Status| iced_aw::menu::Style {
            path_border: Border {
                radius: Radius::new(6.0),
                ..Default::default()
            },
            ..menu_bar::primary(theme, status)
        });

        let settings_row: Row<Message> = Row::new()
            .padding(20)
            .spacing(16)
            .push(Checkbox::new("Add from queue", self.from_queue).on_toggle(Message::FromQueue))
            .push(order_menu)
            .push(word_class_menu);

        let button_row = if self.from_queue {
            Row::new()
                .push(
                    Button::new(Text::new("Ignore").align_x(Horizontal::Center))
                        .width(Length::Fill)
                        .on_press(Message::Ignore),
                )
                .push(
                    Button::new(Text::new("Add").align_x(Horizontal::Center))
                        .width(Length::Fill)
                        .on_press(Message::Add),
                )
                .push(
                    Button::new(Text::new("Blacklist").align_x(Horizontal::Center))
                        .width(Length::Fill)
                        .on_press(Message::Blacklist),
                )
        } else {
            Row::new().push(
                Button::new(Text::new("Add").align_x(Horizontal::Center))
                    .width(Length::Fill)
                    .on_press(Message::Add),
            )
        };

        let column = Column::new()
            .align_x(Alignment::Center)
            .max_width(600)
            .padding(20)
            .spacing(16)
            .push(
                TextInput::new("Russian", &self.russian)
                    .on_input(Message::RussianChanged)
                    .padding(10)
                    .size(32),
            )
            .push(
                TextInput::new("Native", &self.native)
                    .id(INPUT_ID.clone())
                    .on_input(Message::NativeChanged)
                    .padding(10)
                    .size(32)
                    .on_submit(Message::Add),
            )
            .push(button_row)
            .push(settings_row);

        let content: Element<'_, Message> = Container::new(
            Row::new()
                .align_y(Alignment::Center)
                .push(column.width(Length::Fill))
                .push_maybe(entry_scrollable),
        )
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .into();

        content.map(super::Message::Add)
    }
}