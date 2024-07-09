use iced::{
    alignment::{Horizontal, Vertical},
    widget::{text::Shaping, Button, Checkbox, Column, Container, Row, Text, TextInput},
    Alignment, Element, Length, Task,
};
use iced_aw::TabLabel;

use crate::{
    database::{dictionary, schedule},
    dictionary::entry::Entry,
    fsrs::card::Card,
};

use super::Tab;

#[derive(Debug, Clone)]
pub enum Message {
    RussianChanged(String),
    NativeChanged(String),
    DictionaryTimer { version: usize },
    ReadEntries { preloading: bool, word: String },
    EntriesRead { preloading: bool, entries: Vec<Entry> },
    LoadNext,
    Preload,
    Add,
    FromQueue(bool),
    ReadFromQueue,
    QueueRead { lemmas: Vec<String> },
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
        //return Task<Message>
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
                    Task::done(Message::ReadEntries { preloading: false, word: self.russian.clone() } )
                } else {
                    Task::none()
                }
            }
            Message::EntriesRead { preloading, entries } => {
                if preloading {
                    //placeholder string, should take the first word of the expansion
                    self.next_word = Some((String::from("Preloaded Word"), entries));
                } else {
                    self.entries = entries;
                }

                Task::none()
            }
            Message::ReadFromQueue => {
                if self.from_queue {
                    let next_word_is_none = self.next_word.is_none();
                    Task::future(schedule::get_lemmas_queue(self.ignored_from_queue))
                        .then(move |lemmas| match lemmas {
                            Ok(lemmas) => {
                                let mut tasks = vec![Task::done(Message::QueueRead { lemmas })];
                                if next_word_is_none {
                                    tasks.push(Task::done(Message::Preload));
                                }

                                Task::batch(tasks)
                            },
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
                Task::none()
            }
            Message::Blacklist => {
                Task::perform(schedule::blacklist_lemma(self.russian.clone()), |_| {
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
            Message::ReadEntries { preloading, word } => {
                Task::perform(dictionary::read_entries(word), move |entries| {
                    match entries {
                        Ok(entries) => Message::EntriesRead { preloading, entries },
                        Err(e) => Message::Error(e.to_string()),
                    }
                })
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
                    self.russian = lemma.to_owned();
                    self.entries = entries.to_owned();
                    self.next_word = None;
                }

                Task::done(Message::Preload)
            }
            Message::Preload => {
                if self.lemmas.is_empty() {
                    Task::done(Message::ReadFromQueue)
                }
                else {
                    Task::done(
                        Message::ReadEntries { preloading: true, word: self.lemmas.remove(0) }
                    )
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
                    Button::new(Text::new("Ignore").horizontal_alignment(Horizontal::Center))
                        .width(Length::Fill)
                        .on_press(Message::Ignore),
                )
                .push(
                    Button::new(Text::new("Add").horizontal_alignment(Horizontal::Center))
                        .width(Length::Fill)
                        .on_press(Message::Add),
                )
                .push(
                    Button::new(Text::new("Blacklist").horizontal_alignment(Horizontal::Center))
                        .width(Length::Fill)
                        .on_press(Message::Blacklist),
                )
        } else {
            Row::new().push(
                Button::new(Text::new("Add").horizontal_alignment(Horizontal::Center))
                    .width(Length::Fill)
                    .on_press(Message::Add),
            )
        };

        let mut column = Column::new()
            .align_items(Alignment::Center)
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
                    .on_input(Message::NativeChanged)
                    .padding(10)
                    .size(32)
                    .on_submit(Message::Add),
            )
            .push(button_row)
            .push(Checkbox::new("Add From Queue", self.from_queue).on_toggle(Message::FromQueue));

        for entry in &self.entries {
            let etymology = match &entry.etymology {
                Some(etymology) => Some(Text::new(etymology).shaping(Shaping::Advanced)),
                _ => None,
            };
            column = column
                .push(Text::new(format!("{}, {}", entry.word, entry.pos)))
                .push_maybe(etymology)
        }

        let content: Element<'_, Message> = Container::new(column)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into();

        content.map(super::Message::Add)
    }
}
