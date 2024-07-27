use add::{AddTab, Message as AddMessage};
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{Column, Container},
    Element, Length, Task,
};
use iced_aw::{TabLabel, Tabs};
use lemmatize::{LemmatizeTab, Message as LemmatizeMessage};
use main::{MainTab, Message as MainMessage};

use crate::{database, error::Error};

mod add;
mod lemmatize;
mod main;

const HEADER_SIZE: u16 = 32;
const TAB_PADDING: u16 = 16;

pub fn run() -> iced::Result {
    iced::application(App::title, App::update, App::view).run_with(App::new)
}

struct App {
    active_tab: TabId,
    add_tab: AddTab,
    lemmatize_tab: LemmatizeTab,
    main_tab: MainTab,
}

#[derive(Debug)]
enum Message {
    TablesChecked {
        dictionary: bool,
        frequency: bool,
        schedule: bool,
        queue: bool,
    },
    QueueChecked(bool),
    TabSelected(TabId),
    Add(AddMessage),
    Lemmatize(LemmatizeMessage),
    Main(MainMessage),
    Error(Error),
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum TabId {
    Add,
    Lemmatize,
    Main,
}

impl App {
    fn title(&self) -> String {
        String::from("Lern")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TablesChecked { dictionary, frequency, schedule, queue } => {
                self.main_tab.dictionary = dictionary;
                self.main_tab.frequency = frequency;
                self.main_tab.schedule = schedule;
                self.main_tab.queue = queue;

                Task::none()
            }
            Message::QueueChecked(has_items) => {
                self.add_tab.queue_available = has_items;
                Task::none()
            }
            Message::TabSelected(selected) => {
                self.active_tab = selected;
                Task::none()
            }
            Message::Add(message) => match self.add_tab.update(message) {
                add::Action::None => Task::none(),
                add::Action::Run(task) => task.map(Message::Add),
            },
            Message::Lemmatize(message) => match self.lemmatize_tab.update(message) {
                lemmatize::Action::None => Task::none(),
                lemmatize::Action::Run(task) => task.map(Message::Lemmatize),
                lemmatize::Action::Add(task) => task.map(Message::Add),
            },
            Message::Main(message) => match self.main_tab.update(message) {
                main::Action::None => Task::none(),
                main::Action::Run(task) => task.map(Message::Main),
                main::Action::Add(task) => task.map(Message::Add),
            },
            Message::Error(error) => {
                println!("{}", error);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        Tabs::new(Message::TabSelected)
            .push(TabId::Main, self.main_tab.tab_label(), self.main_tab.view())
            .push(TabId::Add, self.add_tab.tab_label(), self.add_tab.view())
            .push(
                TabId::Lemmatize,
                self.lemmatize_tab.tab_label(),
                self.lemmatize_tab.view(),
            )
            .set_active_tab(&self.active_tab)
            .into()
    }
}

trait Tab {
    type Message;

    fn title(&self) -> String;

    fn tab_label(&self) -> TabLabel;

    fn view(&self) -> Element<'_, Self::Message> {
        let column = Column::new()
            .spacing(20)
            .push(self.content())
            .align_x(iced::Alignment::Center);

        Container::new(column)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .padding(TAB_PADDING)
            .into()
    }

    fn content(&self) -> Element<'_, Self::Message>;
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                active_tab: TabId::Add,
                add_tab: AddTab::new(),
                lemmatize_tab: LemmatizeTab::new(),
                main_tab: MainTab::new(),
            },
            Task::batch([
                Task::future(database::check_tables()).then(|res| match res {
                    Err(e) => Task::done(Message::Error(e)),
                    Ok(map) => Task::done(Message::TablesChecked {
                        dictionary: *map.get("dictionary").unwrap(),
                        frequency: *map.get("frequency").unwrap(),
                        schedule: *map.get("cards").unwrap(),
                        queue: *map.get("lemmas").unwrap(),
                    }),
                }),
                Task::future(database::queue::check_queue()).then(|res| match res {
                    Err(e) => Task::done(Message::Error(e)),
                    Ok(has_items) => Task::done(Message::QueueChecked(has_items)),
                })
            ]),
        )
    }
}
