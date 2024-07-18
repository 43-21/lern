use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        text::Shaping, text_input::{focus, Id}, Button, Checkbox, Column, Container, Row, Scrollable, Text, TextInput
    },
    Alignment, Element, Length, Task,
};
use iced_aw::TabLabel;
use once_cell::sync::Lazy;

use crate::{
    database::{dictionary, queue, schedule},
    dictionary::entry::Entry,
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
    Error(String),
}

pub struct AddTab {
    russian: String,
    native: String,
    version: usize,
    entries: Vec<Entry>,
    lemmas: Vec<String>,
    from_queue: bool,
    ignored_from_queue: usize,
    next_word: Option<(String, Vec<Entry>)>,
}

impl AddTab {
    pub fn new() -> AddTab {
        AddTab {
            russian: String::new(),
            native: String::new(),
            version: 0,
            entries: Vec::new(),
            lemmas: Vec::new(),
            from_queue: false,
            ignored_from_queue: 0,
            next_word: None,
        }
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
                if self.native.len() > 0 && self.russian.len() > 0 {
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
                    Task::done(Message::ReadEntries {
                        preloading: false,
                        word: self.russian.clone(),
                    })
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
                            entries.get(0).unwrap().expansion.clone()
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
                }

                if self.russian.is_empty() && preloading {
                    Task::done(Message::LoadNext)
                } else {
                    Task::none()
                }
            }
            Message::ReadFromQueue => {
                if self.from_queue {
                    Task::future(queue::get_lemmas_queue(self.ignored_from_queue)).then(
                        move |lemmas| match lemmas {
                            Ok(lemmas) => Task::done(Message::QueueRead { lemmas }),
                            Err(e) => Task::done(Message::Error(e.to_string())),
                        },
                    )
                } else {
                    Task::none()
                }
            }
            Message::QueueRead { lemmas } => {
                self.lemmas = lemmas;
                if self.lemmas.len() == 0 {
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
                    self.russian = lemma.to_owned();
                    self.entries = entries.to_owned();
                    self.next_word = None;
                }

                Task::batch([Task::done(Message::Preload), focus(INPUT_ID.clone())])
            }
            Message::Preload => {
                if self.lemmas.is_empty() {
                    Task::done(Message::ReadFromQueue)
                } else {
                    Task::done(Message::ReadEntries {
                        preloading: true,
                        word: self.lemmas.remove(0),
                    })
                }
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
            .push(Checkbox::new("Add from queue", self.from_queue).on_toggle(Message::FromQueue));

        let mut entry_column = Column::new()
            .align_x(Alignment::Start)
            .padding(20)
            .spacing(16);

        for entry in &self.entries {
            let etymology = match &entry.etymology {
                Some(etymology) => Some(Text::new(etymology).shaping(Shaping::Advanced)),
                _ => None,
            };

            let word = {
                if let Some(expansion) = &entry.expansion {
                    expansion
                } else {
                    &entry.word
                }
            };

            if !entry.pronunciations.is_empty() {
                entry_column = entry_column.push(Text::new("Pronunciation"));
            }

            for pronunciation in &entry.pronunciations {
                let tag_string = {
                    if pronunciation.tags.is_empty() {
                        String::new()
                    } else {
                        format!(" ({})", pronunciation.tags.join(", "))
                    }
                };
                entry_column = entry_column.push(
                    Text::new(format!("  {}{}", pronunciation.ipa, tag_string))
                        .shaping(Shaping::Advanced),
                )
            }

            entry_column = entry_column
                .push(Text::new(format!("{}, {}", word, entry.pos)).shaping(Shaping::Advanced))
                .push_maybe(etymology);

            for (i, sense) in entry.senses.iter().enumerate() {
                let tag_string = {
                    if sense.tags.is_empty() {
                        String::new()
                    } else {
                        format!(" ({})", sense.tags.join(", "))
                    }
                };
                entry_column = entry_column.push(
                    Text::new(format!("    {}. {}{}", i + 1, sense.sense, tag_string))
                        .shaping(Shaping::Advanced),
                );

                for example in &sense.examples {
                    let translation = if let Some(example) = &example.english {
                        format!(" - {}", example)
                    } else {
                        String::new()
                    };
                    entry_column = entry_column.push(
                        Text::new(format!("        {}{}", example.text, translation))
                            .shaping(Shaping::Advanced),
                    );
                }
            }
        }

        let entry_scrollable = if self.entries.is_empty() {
            None
        } else {
            Some(Scrollable::new(entry_column).width(Length::Fill))
        };

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
