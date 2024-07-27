use std::path::PathBuf;

use iced::{
    alignment::{Horizontal, Vertical},
    widget::{text_editor, Button, Checkbox, Column, Container, Row, Text},
    Alignment, Element, Length, Task,
};
use iced_aw::TabLabel;
use rfd::AsyncFileDialog;

use crate::dictionary::{lemmatize, lemmatize_from_file};

use super::Tab;

pub struct LemmatizeTab {
    content: text_editor::Content,
    is_dirty: bool,
    add_sentences: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    ActionPerformed(text_editor::Action),
    Lemmatize,
    FromFile,
    FileSet { path: Option<PathBuf> },
    AddSentences(bool),
    Lemmatized,
    Error(String),
}

pub enum Action {
    None,
    Run(Task<Message>),
    Add(Task<super::AddMessage>),
}

impl LemmatizeTab {
    pub fn new() -> LemmatizeTab {
        LemmatizeTab {
            content: text_editor::Content::new(),
            is_dirty: false,
            add_sentences: true,
        }
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::ActionPerformed(action) => {
                self.is_dirty = self.is_dirty || action.is_edit();
                self.content.perform(action);
                Action::None
            }
            Message::Lemmatize => {
                let text = self.content.text();
                self.content = text_editor::Content::new();
                self.is_dirty = false;

                Action::Run(Task::future(lemmatize(text, self.add_sentences)).then(|result| match result {
                    Ok(()) => Task::done(Message::Lemmatized),
                    Err(e) => Task::done(Message::Error(e.to_string())),
                }))
            }
            Message::FromFile => Action::Run(Task::perform(
                AsyncFileDialog::new()
                    .set_title("From file")
                    .add_filter("text", &["txt", "srt"])
                    .pick_file(),
                |file_handle| Message::FileSet {
                    path: file_handle.map(|file_handle| file_handle.into()),
                },
            )),
            Message::FileSet { path } => {
                if let Some(path) = path {
                    Action::Run(Task::future(lemmatize_from_file(path, self.add_sentences)).then(|result| {
                        match result {
                            Ok(()) => Task::none(),
                            Err(e) => Task::done(Message::Error(e.to_string())),
                        }
                    }))
                } else {
                    Action::None
                }
            }
            Message::AddSentences(value) => {
                self.add_sentences = value;
                Action::None
            }
            Message::Lemmatized => {
                Action::Add(Task::done(super::AddMessage::QueueInsertion))
            }
            Message::Error(e) => {
                println!("{e}");
                Action::None
            }
        }
    }
}

impl Tab for LemmatizeTab {
    type Message = super::Message;

    fn title(&self) -> String {
        String::from("Lemmatize")
    }

    fn tab_label(&self) -> TabLabel {
        TabLabel::Text(self.title())
    }

    fn content(&self) -> iced::Element<'_, Self::Message> {
        let content: Element<'_, Message> = Container::new(
            Row::new()
                .align_y(Alignment::Center)
                .padding(20)
                .spacing(16)
                .push(
                    Column::new()
                        .align_x(Alignment::Center)
                        .padding(20)
                        .spacing(8)
                        .push(
                            Checkbox::new("Add sentences", self.add_sentences)
                                .on_toggle(Message::AddSentences),
                        )
                        .push(
                            text_editor(&self.content)
                                .height(Length::Fill)
                                .on_action(Message::ActionPerformed),
                        )
                        .push(Button::new(Text::new("From file")).on_press(Message::FromFile)),
                )
                .push(Button::new(Text::new("Lemmatize")).on_press(Message::Lemmatize)),
        )
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .into();

        content.map(super::Message::Lemmatize)
    }
}
