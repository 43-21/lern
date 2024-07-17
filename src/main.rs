#![windows_subsystem = "windows"]
#![allow(dead_code)]

mod database;
mod fsrs;
mod dictionary;
mod gui;

fn main() {
    gui::run().unwrap();
}