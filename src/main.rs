#![allow(dead_code)]

mod database;
mod fsrs;
mod dictionary;
mod gui;

fn main() {
    gui::run().unwrap();
    // tokio::runtime::Builder::new_multi_thread()
    // .enable_all()
    // .build()
    // .unwrap()
    // .block_on(
    //     async {
    //         database::create_dictionary().await.unwrap();
    //         database::create_schedule().await.unwrap();
    //     }
    // )
}