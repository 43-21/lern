// #![windows_subsystem = "windows"]
#![allow(dead_code)]

use error::Error;

mod database;
mod dictionary;
mod error;
mod fsrs;
mod gui;

pub type Result<T, E = Error> = std::result::Result<T, E>;

fn main() {
    gui::run().unwrap();
}
