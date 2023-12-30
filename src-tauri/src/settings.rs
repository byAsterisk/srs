/*
(c) Matthew Boyer, 2023.

This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at https://mozilla.org/MPL/2.0/.

This Source Code Form is "Incompatible With Secondary Licenses", as
defined by the Mozilla Public License, v. 2.0.
*/

use dirs;
use serde_json;
use serde_json::{json, Value};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Mutex;

pub struct Settings { settings: Box<Mutex<Value>> }

impl Settings {
    pub fn default() -> Settings {
        let mut content = String::default();
        fs::File::open(Self::get_path()).unwrap().read_to_string(&mut content).unwrap();
        let val: Value = serde_json::from_str(&content).unwrap();
        Settings { settings: Box::new(Mutex::from(val.clone())) }
    }

    fn get_path() -> String {
        let json_path = dirs::data_dir().unwrap().to_str().unwrap().to_string() + "/srs/config.json";
        let json_file = Path::new(&json_path);
        if !json_file.exists() {
            let json_dir = Path::new(&json_path).parent().unwrap();
            fs::create_dir_all(json_dir).unwrap();
            fs::File::create(json_file).unwrap().write(json!({
                "new_cards": 20
            }).to_string().as_bytes()).unwrap();
        }

        json_path
    }

    pub fn get(&self) -> Value { self.settings.lock().unwrap().clone().take() }

    pub fn get_from_file(key: &str) -> Value {
        let mut content = String::default();
        fs::File::open(Self::get_path()).unwrap().read_to_string(&mut content).unwrap();
        serde_json::from_str::<Value>(&content).unwrap()[key].clone()
    }

    pub fn set(&self, value: Value) { *(self.settings.lock().unwrap()) = value; }

    pub fn save(&self) {
        fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(Self::get_path()).unwrap()
            .write_all(&*self.settings.lock().unwrap().to_string().as_bytes())
            .unwrap();
    }
}