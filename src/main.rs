// #![windows_subsystem = "windows"]
#![allow(dead_code)]

mod database;
mod dictionary;
mod fsrs;
mod gui;

fn main() {
    gui::run().unwrap();
}
