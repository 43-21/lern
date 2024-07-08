use iced::{
    alignment::{Horizontal, Vertical},
    widget::{text_editor, Button, Container, Row, Text},
    Alignment, Element, Length, Task
};
use iced_aw::TabLabel;

use crate::{database::schedule, dictionary::lemmatize};

use super::Tab;

pub struct LemmatizeTab {
    content: text_editor::Content,
    is_dirty: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    ActionPerformed(text_editor::Action),
    Lemmatize, LemmatizeResults(Vec<String>),
    LemmasInserted, LemmasOrdered,
    Error(String),
}

impl LemmatizeTab {
    pub fn new() -> LemmatizeTab {
        LemmatizeTab {
            content: text_editor::Content::new(),
            is_dirty: false,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ActionPerformed(action) => {
                self.is_dirty = self.is_dirty || action.is_edit();
                self.content.perform(action);
                Task::none()
            }
            Message::Lemmatize => {
                Task::perform(lemmatize(self.content.text()), |result| {
                    Message::LemmatizeResults(result.unwrap())
                })
            }
            Message::LemmatizeResults(lemmas) => {
                Task::batch(vec![
                    Task::perform(schedule::insert_lemmas(lemmas), |res| {
                        match res {
                            Ok(_) => Message::LemmasInserted,
                            Err(e) => Message::Error(e.to_string()),
                        }
                    }),
                ])
            }
            Message::LemmasInserted => {
                Task::none()
            }
            Message::LemmasOrdered => {
                Task::none()
            }
            Message::Error(e) => {
                println!("{e}");
                Task::none()
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
                .align_items(Alignment::Center)
                .padding(20)
                .spacing(16)
                .push(
                    text_editor(&self.content)
                    .height(Length::Fill)
                    .on_action(Message::ActionPerformed)
                )
                .push(
                    Button::new(Text::new("Lemmatize"))
                    .on_press(Message::Lemmatize)
                )
        )
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .into();

        content.map(super::Message::Lemmatize)
    }
}