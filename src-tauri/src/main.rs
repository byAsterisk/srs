/*
(c) Matthew Boyer, 2023.

This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at https://mozilla.org/MPL/2.0/.

This Source Code Form is "Incompatible With Secondary Licenses", as
defined by the Mozilla Public License, v. 2.0.
*/

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cards;
mod settings;

use chrono::Utc;
use fsrs::{FSRS, Rating};
use serde_json::{json, Value};
use tauri::State;

fn main() {
    tauri::Builder::default()
        .manage(settings::Settings::default())
        .manage(cards::Cards::default())
        .invoke_handler(tauri::generate_handler![
            card_count,
            next_card, update_card,
            get_settings, set_settings,
            get_decks, new_deck, import_deck, export_deck, rename_deck, delete_deck,
            get_deck, new_card, edit_card, reset_card, delete_card,
            exit
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command] fn card_count(cards: State<cards::Cards>) -> i64 { cards.card_count() }

#[tauri::command]
fn next_card(cards: State<cards::Cards>) -> Value {
    let card = cards.current_card();
    match card {
        Ok(_) => json!([card.clone().unwrap().2, card.clone().unwrap().3]),
        Err(_) => json!([])
    }
}

#[tauri::command]
fn update_card(rating: i8, cards: State<cards::Cards>) {
    let card = cards.current_card().unwrap();
    cards.update_card(card.clone().0, card.clone().1, &FSRS::default().schedule(card.clone().4, Utc::now()).select_card(match rating {
        1 => Rating::Again,
        2 => Rating::Hard,
        3 => Rating::Good,
        4 => Rating::Easy,
        _ => panic!()
    }));
}

#[tauri::command] fn get_settings (settings: State<settings::Settings>) -> Value { settings.get() }

#[tauri::command]
fn set_settings(settings: State<settings::Settings>, value: Value) {
    settings.set(value);
    settings.save();
}

#[tauri::command] fn get_decks() -> Vec<String> { cards::Cards::get_decks() }
#[tauri::command] fn new_deck(deck: String) { cards::Cards::new_deck(deck); }
#[tauri::command] fn import_deck(path: String, cards: State<cards::Cards>) { cards.import_deck(path).unwrap(); }
#[tauri::command] fn export_deck(deck: String, path: String) { cards::Cards::export_deck(deck, path); }
#[tauri::command] fn rename_deck(deck: String, name: String, cards: State<cards::Cards>) { cards.rename_deck(deck, name); }
#[tauri::command] fn delete_deck(deck: String, cards: State<cards::Cards>) { cards.delete_deck(deck); }

#[tauri::command] fn get_deck(deck: String) -> Vec<(String, i64, String, String, Value)> { cards::Cards::get_deck(deck) }
#[tauri::command] fn new_card(deck: String, cards: State<cards::Cards>) -> i64 { cards.new_card(deck, String::default(), String::default()).unwrap() }
#[tauri::command] fn edit_card(deck: String, id: i64, obverse: String, reverse: String, cards: State<cards::Cards>) { cards.edit_card(deck, id, obverse, reverse); }
#[tauri::command] fn reset_card(deck: String, id: i64, cards: State<cards::Cards>) { cards.reset_card(deck, id); }
#[tauri::command] fn delete_card(deck: String, id: i64, cards: State<cards::Cards>) { cards.delete_card(deck, id); }

#[tauri::command] fn exit() { std::process::exit(0); }