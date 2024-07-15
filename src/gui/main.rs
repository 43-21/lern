use std::path::PathBuf;

use iced::{
    alignment::{Horizontal, Vertical},
    widget::{text::Shaping, Button, Column, Container, Row, Text},
    Alignment, Element, Task,
};
use iced_aw::TabLabel;
use rfd::AsyncFileDialog;

use crate::database::{self, schedule};

use super::Tab;

pub struct MainTab {
    wiktionary_path: Option<PathBuf>,
    frequency_path: Option<PathBuf>,
    dictionary: bool,
    schedule: bool,
    frequency: bool,
    queue: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    Error(String),
    SetWiktionaryFile,
    SetFrequencyFile,
    WiktionaryFileSet { path: Option<PathBuf> },
    FrequencyFileSet { path: Option<PathBuf> },
    CreateSchedule,
    CreateQueue,
    CreateDictionary,
    CreateFrequency,
    ScheduleCreated,
    QueueCreated,
    DictionaryCreated,
    FrequencyCreated,
    SetExportLocation,
    Export { path: Option<PathBuf> },
    Exported,
}

impl MainTab {
    pub fn new() -> MainTab {
        MainTab {
            wiktionary_path: None,
            frequency_path: None,
            dictionary: false,
            schedule: false,
            frequency: false,
            queue: false,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Error(e) => {
                println!("{e}");
                Task::none()
            }
            Message::SetWiktionaryFile => Task::perform(
                AsyncFileDialog::new()
                    .set_title("Wiktionary")
                    .add_filter("JSON Lines", &["jsonl"])
                    .pick_file(),
                |file_handle| Message::WiktionaryFileSet {
                    path: match file_handle {
                        Some(file_handle) => Some(file_handle.into()),
                        None => None,
                    },
                },
            ),
            Message::SetFrequencyFile => Task::perform(
                AsyncFileDialog::new()
                    .set_title("Frequency")
                    .add_filter("text", &["txt"])
                    .pick_file(),
                |file_handle| Message::FrequencyFileSet {
                    path: match file_handle {
                        Some(file_handle) => Some(file_handle.into()),
                        None => None,
                    },
                },
            ),
            Message::WiktionaryFileSet { path } => {
                if path.is_some() {
                    self.wiktionary_path = path;
                }
                Task::none()
            }
            Message::FrequencyFileSet { path } => {
                if path.is_some() {
                    self.frequency_path = path;
                }
                Task::none()
            }
            Message::CreateSchedule => {
                Task::perform(database::create_schedule(), |res| match res {
                    Err(e) => Message::Error(e.to_string()),
                    Ok(()) => Message::ScheduleCreated,
                })
            }
            Message::CreateQueue => Task::perform(database::create_queue(), |res| match res {
                Err(e) => Message::Error(e.to_string()),
                Ok(()) => Message::QueueCreated,
            }),
            Message::CreateDictionary => Task::perform(
                database::create_dictionary(self.wiktionary_path.clone().unwrap()),
                |res| match res {
                    Err(e) => Message::Error(e.to_string()),
                    Ok(()) => Message::DictionaryCreated,
                },
            ),
            Message::CreateFrequency => Task::perform(
                database::create_frequency(self.frequency_path.clone().unwrap()),
                |res| match res {
                    Err(e) => Message::Error(e.to_string()),
                    Ok(()) => Message::FrequencyCreated,
                },
            ),
            Message::ScheduleCreated => {
                self.schedule = true;
                Task::none()
            }
            Message::DictionaryCreated => {
                self.dictionary = true;
                Task::none()
            }
            Message::QueueCreated => {
                self.queue = true;
                Task::none()
            }
            Message::FrequencyCreated => {
                self.frequency = true;
                Task::none()
            }
            Message::SetExportLocation => Task::perform(
                AsyncFileDialog::new()
                    .set_title("Export")
                    .add_filter("text", &["txt"])
                    .set_directory("Downloads")
                    .set_file_name("RussianDeck")
                    .save_file(),
                |file_handle| Message::Export {
                    path: match file_handle {
                        Some(file_handle) => Some(file_handle.into()),
                        None => None,
                    },
                },
            ),
            Message::Export { path } => match path {
                Some(path) => Task::perform(schedule::export(path), |res| match res {
                    Ok(()) => Message::Exported,
                    Err(e) => Message::Error(e.to_string()),
                }),
                None => Task::none(),
            },
            Message::Exported => {
                println!("exported!");
                Task::none()
            }
        }
    }
}

impl Tab for MainTab {
    type Message = super::Message;

    fn title(&self) -> String {
        String::from("Main")
    }

    fn tab_label(&self) -> TabLabel {
        TabLabel::Text(self.title())
    }

    fn content(&self) -> iced::Element<'_, Self::Message> {
        let frequency_msg = {
            if self.frequency_path.is_some() && self.dictionary {
                Some(Message::CreateFrequency)
            } else {
                None
            }
        };
        let dictionary_msg: Option<Message> = {
            if self.wiktionary_path.is_some() {
                Some(Message::CreateDictionary)
            } else {
                None
            }
        };

        let dictionary = {
            if self.dictionary {
                Some(Text::new("✓").shaping(Shaping::Advanced))
            } else {
                None
            }
        };

        let frequency = {
            if self.frequency {
                Some(Text::new("✓").shaping(Shaping::Advanced))
            } else {
                None
            }
        };

        let schedule = {
            if self.schedule {
                Some(Text::new("✓").shaping(Shaping::Advanced))
            } else {
                None
            }
        };

        let queue = {
            if self.queue {
                Some(Text::new("✓").shaping(Shaping::Advanced))
            } else {
                None
            }
        };

        let file_row = Row::new()
            .align_y(Alignment::Center)
            .padding(20)
            .spacing(16)
            .push(
                Button::new(Text::new("Load dictionary file")).on_press(Message::SetWiktionaryFile),
            )
            .push(
                Button::new(Text::new("Load frequency file")).on_press(Message::SetFrequencyFile),
            );

        let create_row = Row::new()
            .align_y(Alignment::Center)
            .padding(20)
            .spacing(16)
            .push_maybe(dictionary)
            .push(Button::new(Text::new("Create dictionary")).on_press_maybe(dictionary_msg))
            .push(Button::new(Text::new("Create frequencies")).on_press_maybe(frequency_msg))
            .push_maybe(frequency);

        let clear_row = Row::new()
            .align_y(Alignment::Center)
            .padding(20)
            .spacing(16)
            .push_maybe(schedule)
            .push(Button::new(Text::new("Create schedule")).on_press(Message::CreateSchedule))
            .push(Button::new(Text::new("Create queue")).on_press(Message::CreateQueue))
            .push_maybe(queue);

        let content: Element<'_, Message> = Container::new(
            Column::new()
                .align_x(Alignment::Center)
                .padding(20)
                .spacing(16)
                .push(file_row)
                .push(create_row)
                .push(clear_row)
                .push(
                    Button::new(Text::new("Export to Anki")).on_press(Message::SetExportLocation),
                ),
        )
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .into();

        content.map(super::Message::Main)
    }
}
