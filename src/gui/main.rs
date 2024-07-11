use std::path::PathBuf;

use iced::{
    alignment::{Horizontal, Vertical},
    widget::{Button, Column, Container, Row, Text},
    Alignment, Element, Task
};
use iced_aw::TabLabel;

use super::Tab;

pub struct MainTab {
    file: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Error(String),
    OpenFile,
    CreateDatabase,
    ResetSchedule, ResetQueue,
    UpdateDictionary,
}

impl MainTab {
    pub fn new() -> MainTab {
        MainTab {
            file: None,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Error(e) => {
                println!("{e}");
                Task::none()
            }
            Message::OpenFile => {
                Task::none()
            }
            Message::CreateDatabase => {
                Task::none()
            }
            Message::ResetSchedule => {
                Task::none()
            }
            Message::ResetQueue => {
                Task::none()
            }
            Message::UpdateDictionary => {
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
        let database_row = Row::new()
        .align_items(Alignment::Center)
        .padding(20)
        .spacing(16)
        .push(
            Button::new(Text::new("Set Dictionary JSON File"))
            .on_press(Message::OpenFile)
        )
        .push(
            Button::new(Text::new("Create Database"))
            .on_press(Message::CreateDatabase)
        );


        let clear_row = Row::new()
        .align_items(Alignment::Center)
        .padding(20)
        .spacing(16)
        .push(
            Button::new(Text::new("Clear Schedule"))
            .on_press(Message::ResetSchedule)
        )
        .push(
            Button::new(Text::new("Clear Queue"))
            .on_press(Message::ResetQueue)
        );

        let content: Element<'_, Message> = Container::new(
            Column::new()
                .align_items(Alignment::Center)
                .padding(20)
                .spacing(16)
                .push(database_row)
                .push(
                    Button::new(Text::new("Update Dictionary"))
                    .on_press(Message::UpdateDictionary)
                )
                .push(clear_row)
        )
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .into();

        content.map(super::Message::Main)
    }
}