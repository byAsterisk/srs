/*
(c) Matthew Boyer, 2023.

This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at https://mozilla.org/MPL/2.0/.

This Source Code Form is "Incompatible With Secondary Licenses", as
defined by the Mozilla Public License, v. 2.0.
*/

use chrono::{DateTime, Utc};
use dirs;
use fsrs::{Card, Rating, State as CardState, ReviewLog, to_json};
use sqlite::{Connection, State as DBState, Statement};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Mutex;
use crate::settings;

pub struct Cards { pub cards: Mutex<Vec<(String, i64, String, String, Card)>> }

impl Cards {
    pub fn default() -> Cards { Cards { cards: Mutex::from(Self::get_cards()) } }

    //noinspection DuplicatedCode
    fn get_path() -> String {
        let db_path = dirs::data_dir().unwrap().to_str().unwrap().to_string() + "/srs/srs.sqlite";
        let db_file = Path::new(&db_path);
        if !db_file.exists() {
            let db_dir = Path::new(&db_path).parent().unwrap();
            fs::create_dir_all(db_dir).unwrap();
            fs::File::create(db_file).unwrap();
        }

        db_path
    }

    fn get_cards() -> Vec<(String, i64, String, String, Card)> {
        let connection = Connection::open(Self::get_path()).unwrap();
        let decks: Vec<String> = Self::get_decks();
        let mut cards: Vec<(String, i64, String, String, Card)> = Vec::new();

        for deck in decks {
            let mut new_cards: i64 = 0;
            let mut statement = connection.prepare(format!("SELECT FIRST_STUDY FROM \"{}\"", deck.replace(r#"""#, r#""""#))).unwrap();
            while let Ok(DBState::Row) = statement.next() {
                if statement.read::<String, _>("FIRST_STUDY").is_ok() {
                    if <DateTime<Utc> as PartialOrd<DateTime<Utc>>>::gt(
                        &DateTime::from(DateTime::parse_from_rfc3339(&statement.read::<String, _>("FIRST_STUDY").unwrap()).unwrap()),
                        &DateTime::from_naive_utc_and_offset(Utc::now().date_naive().and_time(chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap()), Utc)
                    ) { new_cards += 1; }
                }
            }
            let mut statement = connection.prepare(format!("SELECT ROWID, * FROM \"{}\" ORDER BY DUE", deck.replace(r#"""#, r#""""#))).unwrap();
            while let Ok(DBState::Row) = statement.next() {
                if <DateTime<Utc> as PartialOrd<DateTime<Utc>>>::le(
                    &DateTime::from(DateTime::parse_from_rfc3339(&statement.read::<String, _>("DUE").unwrap()).unwrap()),
                    &Utc::now()
                ) && (statement.read::<i64, _>("STATE").unwrap() != 1 || new_cards < settings::Settings::get_from_file("new_cards").as_i64().unwrap()) { Self::add_card_to_vec(&mut cards, &mut statement, &deck); }
                if statement.read::<i64, _>("STATE").unwrap() == 1 {new_cards += 1}
            }
        }

        cards
    }

    fn refresh(&self) { *(self.cards.lock().unwrap()) = Self::get_cards(); }

    fn add_card_to_vec(cards: &mut Vec<(String, i64, String, String, Card)>, statement: &mut Statement, deck: &String) {
        cards.push((deck.clone(), statement.read::<i64, _>("rowid").unwrap(), statement.read::<String, _>("OBVERSE").unwrap().as_str().to_string(), statement.read::<String, _>("REVERSE").unwrap().as_str().to_string(), Card {
            due: DateTime::from(DateTime::parse_from_rfc3339(&statement.read::<String, _>("DUE").unwrap()).unwrap()),
            stability: statement.read::<f64, _>("STABILITY").unwrap() as f32,
            difficulty: statement.read::<f64, _>("DIFFICULTY").unwrap() as f32,
            elapsed_days: statement.read::<i64, _>("ELAPSED_DAYS").unwrap(),
            scheduled_days: statement.read::<i64, _>("SCHEDULED_DAYS").unwrap(),
            reps: statement.read::<i64, _>("REPS").unwrap() as i32,
            lapses: statement.read::<i64, _>("LAPSES").unwrap() as i32,
            state: match statement.read::<i64, _>("STATE").unwrap() {
                1 => CardState::New,
                2 => CardState::Learning,
                3 => CardState::Review,
                4 => CardState::Relearning,
                _ => panic!()
            },
            last_review: DateTime::from(DateTime::parse_from_rfc3339(&statement.read::<String, _>("LAST_REVIEW").unwrap()).unwrap()),
            previous_state: match statement.read::<i64, _>("PREVIOUS_STATE").unwrap() {
                1 => CardState::New,
                2 => CardState::Learning,
                3 => CardState::Review,
                4 => CardState::Relearning,
                _ => panic!()
            },
            log: match statement.read::<i64, _>("LOG_RATING").unwrap() {  // statement.read::<i64, _> returns Ok(0) on a null value ?????
                0 => None,
                _ => Some(ReviewLog {
                    rating: match statement.read::<i64, _>("LOG_RATING").unwrap() {
                        1 => Rating::Again,
                        2 => Rating::Hard,
                        3 => Rating::Good,
                        4 => Rating::Easy,
                        _ => panic!()
                    },
                    elapsed_days: statement.read::<i64, _>("LOG_ELAPSED_DAYS").unwrap(),
                    scheduled_days: statement.read::<i64, _>("LOG_SCHEDULED_DAYS").unwrap(),
                    state: match statement.read::<i64, _>("LOG_STATE").unwrap() {
                        1 => CardState::New,
                        2 => CardState::Learning,
                        3 => CardState::Review,
                        4 => CardState::Relearning,
                        _ => panic!()
                    },
                    reviewed_date: DateTime::from(DateTime::parse_from_rfc3339(&statement.read::<String, _>("LOG_REVIEWED_DATE").unwrap()).unwrap()),
                })
            }
        }));
    }

    pub fn card_count(&self) -> i64 { self.cards.lock().unwrap().len() as i64 }

    pub fn current_card(&self) -> Result<(String, i64, String, String, Card), ()> { Ok(self.cards.lock().unwrap().first().ok_or(())?.to_owned()) }

    pub fn update_card(&self, deck: String, id: i64, card: &Card) {
        let connection = Connection::open(Self::get_path()).unwrap();
        connection.execute(format!("UPDATE \"{}\" SET DUE = '{}', STABILITY = {}, DIFFICULTY = {}, ELAPSED_DAYS = {}, SCHEDULED_DAYS = {}, REPS = {}, LAPSES = {}, STATE = {}, LAST_REVIEW = '{}', PREVIOUS_STATE = {}, LOG_RATING = {}, LOG_ELAPSED_DAYS = {}, LOG_SCHEDULED_DAYS = {}, LOG_STATE = {}, LOG_REVIEWED_DATE = '{}' WHERE ROWID = {}",
                                   deck.replace(r#"""#, r#""""#),
                                   card.due.to_rfc3339(),
                                   card.stability,
                                   card.difficulty,
                                   card.elapsed_days,
                                   card.scheduled_days,
                                   card.reps,
                                   card.lapses,
                                   match card.state {
                                       CardState::New => 1,
                                       CardState::Learning => 2,
                                       CardState::Review => 3,
                                       CardState::Relearning => 4
                                   },
                                   card.last_review.to_rfc3339(),
                                   match card.previous_state {
                                       CardState::New => 1,
                                       CardState::Learning => 2,
                                       CardState::Review => 3,
                                       CardState::Relearning => 4
                                   },
                                   match card.log.clone().unwrap().rating {
                                       Rating::Again => 1,
                                       Rating::Hard => 2,
                                       Rating::Good => 3,
                                       Rating::Easy => 4
                                   },
                                   card.log.clone().unwrap().elapsed_days,
                                   card.log.clone().unwrap().scheduled_days,
                                   match card.log.clone().unwrap().state {
                                       CardState::New => 1,
                                       CardState::Learning => 2,
                                       CardState::Review => 3,
                                       CardState::Relearning => 4
                                   },
                                   card.log.clone().unwrap().reviewed_date.to_rfc3339(),
                                   id
        )).unwrap();

        if connection.prepare(format!("SELECT FIRST_STUDY FROM \"{}\" WHERE ROWID = {}", deck.replace(r#"""#, r#""""#), id)).unwrap().read::<String, _>("FIRST_STUDY").is_err() {
            connection.execute(format!("UPDATE \"{}\" SET FIRST_STUDY = '{}' WHERE ROWID = {}", deck.replace(r#"""#, r#""""#), Utc::now().to_rfc3339(), id)).unwrap();
        }

        self.refresh();
    }

    pub fn get_decks() -> Vec<String> {
        let connection = Connection::open(Self::get_path()).unwrap();
        let mut statement = connection.prepare("SELECT * FROM sqlite_master WHERE TYPE='table'").unwrap();
        let mut decks: Vec<String> = Vec::new();
        while let Ok(DBState::Row) = statement.next() { decks.push(statement.read::<String, _>("tbl_name").unwrap()); }

        decks
    }

    pub fn new_deck(deck: String) { Connection::open(Self::get_path()).unwrap().execute(format!("CREATE TABLE \"{}\" (OBVERSE TEXT, REVERSE TEXT, DUE TEXT, STABILITY REAL, DIFFICULTY REAL, ELAPSED_DAYS INTEGER, SCHEDULED_DAYS INTEGER, REPS INTEGER, LAPSES INTEGER, STATE INTEGER, LAST_REVIEW TEXT, PREVIOUS_STATE INTEGER, LOG_RATING INTEGER, LOG_ELAPSED_DAYS INTEGER, LOG_SCHEDULED_DAYS INTEGER, LOG_STATE INTEGER, LOG_REVIEWED_DATE TEXT, FIRST_STUDY TEXT)", deck.replace(r#"""#, r#""""#))).unwrap(); }

    pub fn import_deck(&self, path: String) -> Result<(), ()> {
        let mut content = String::default();
        fs::File::open(&path).unwrap().read_to_string(&mut content).unwrap();
        let card_vec: Vec<serde_json::Value> = serde_json::from_str(content.as_str()).unwrap();
        let connection = Connection::open(Self::get_path()).unwrap();
        let structure = "\n(
    OBVERSE            TEXT,
    REVERSE            TEXT,
    DUE                TEXT,
    STABILITY          REAL,
    DIFFICULTY         REAL,
    ELAPSED_DAYS       INTEGER,
    SCHEDULED_DAYS     INTEGER,
    REPS               INTEGER,
    LAPSES             INTEGER,
    STATE              INTEGER,
    LAST_REVIEW        TEXT,
    PREVIOUS_STATE     INTEGER,
    LOG_RATING         INTEGER,
    LOG_ELAPSED_DAYS   INTEGER,
    LOG_SCHEDULED_DAYS INTEGER,
    LOG_STATE          INTEGER,
    LOG_REVIEWED_DATE  TEXT,
    FIRST_STUDY        TEXT
)";
        let file_name = Path::new(path.as_str()).file_name().ok_or(())?.to_str().unwrap();
        let mut deck_name = String::default();
        if connection.execute(format!("CREATE TABLE \"{}\"", file_name.replace(r#"""#, r#""""#)) + structure).is_err() {
            loop {
                let mut att: u64 = 1;
                if connection.execute(format!("CREATE TABLE \"{}({})\"", file_name.replace(r#"""#, r#""""#), att) + structure).is_ok() {
                    deck_name = format!("{}({})", file_name, att);
                    break;
                } else { att += 1; }
            }
        } else { deck_name = file_name.to_string(); }

        for card in card_vec { self.new_card(deck_name.to_string(), str::replace(card[0].as_str().unwrap(), "\\n", "\n").to_string(), str::replace(card[1].as_str().unwrap(), "\\n", "\n").to_string())?; }
        self.refresh();
        Ok(())
    }

    //noinspection DuplicatedCode
    pub fn export_deck(deck: String, path: String) {
        let deck_full = Self::get_deck(deck);
        let mut deck: Vec<serde_json::Value> = Vec::new();
        for card in deck_full { deck.push(serde_json::json!([card.clone().2, card.clone().3])) }
        let file = Path::new(&path);
        if !file.exists() {
            let dir = Path::new(&path).parent().unwrap();
            fs::create_dir_all(dir).unwrap();
            fs::File::create(file).unwrap();
        }

        fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path).unwrap()
            .write_all(serde_json::Value::Array(deck).to_string().as_bytes())
            .unwrap();
    }

    pub fn rename_deck(&self, deck: String, name: String) {
        Connection::open(Self::get_path()).unwrap().execute(format!("ALTER TABLE \"{}\" RENAME TO \"{}\"", deck.replace(r#"""#, r#""""#), name.replace(r#"""#, r#""""#))).unwrap();
        self.refresh();
    }

    pub fn delete_deck(&self, deck: String) {
        Connection::open(Self::get_path()).unwrap().execute(format!("DROP TABLE \"{}\"", deck.replace(r#"""#, r#""""#))).unwrap();
        self.refresh();
    }

    pub fn get_deck(deck: String) -> Vec<(String, i64, String, String, serde_json::Value)>{
        let connection = Connection::open(Self::get_path()).unwrap();
        let mut statement = connection.prepare(format!("SELECT ROWID, * FROM \"{}\"", deck.replace(r#"""#, r#""""#))).unwrap();
        let mut cards: Vec<(String, i64, String, String, Card)> = Vec::new();
        while let Ok(DBState::Row) = statement.next() { Self::add_card_to_vec(&mut cards, &mut statement, &deck); }
        let mut deck: Vec<(String, i64, String, String, serde_json::Value)> = Vec::new();
        for card in cards { deck.push((card.clone().0, card.clone().1, card.clone().2, card.clone().3, to_json(card.clone().4))); }

        deck
    }

    pub fn new_card(&self, deck: String, obverse: String, reverse: String) -> Result<i64, ()> {
        let card_json = to_json(Card::new());
        let connection = Connection::open(Self::get_path()).unwrap();
        let mut statement = connection.prepare(
            format!("INSERT INTO \"{}\" (OBVERSE, REVERSE, DUE, STABILITY, DIFFICULTY, ELAPSED_DAYS, SCHEDULED_DAYS, REPS, LAPSES, STATE, LAST_REVIEW, PREVIOUS_STATE) VALUES (\"{}\", \"{}\", '{}', {}, {}, {}, {}, {}, {}, {}, '{}', {}) RETURNING ROWID",
                    deck.replace(r#"""#, r#""""#),
                    obverse.replace(r#"""#, r#""""#),
                    reverse.replace(r#"""#, r#""""#),
                    card_json["due"].as_str().unwrap().to_string(),
                    card_json["stability"],
                    card_json["difficulty"],
                    card_json["elapsed_days"],
                    card_json["scheduled_days"],
                    card_json["reps"],
                    card_json["lapses"],
                    card_json["state"],
                    card_json["last_review"].as_str().unwrap().to_string(),
                    card_json["previous_state"]
            )
        ).unwrap();
        statement.next().unwrap();
        self.refresh();
        Ok(statement.read::<i64, _>("rowid").unwrap())
    }

    pub fn edit_card(&self, deck: String, id: i64, obverse: String, reverse: String) {
        Connection::open(Self::get_path()).unwrap().execute(format!("UPDATE \"{}\" SET OBVERSE = \"{}\", REVERSE = \"{}\" WHERE ROWID = {}",
            deck.replace(r#"""#, r#""""#),
            obverse.replace(r#"""#, r#""""#),
            reverse.replace(r#"""#, r#""""#),
            id
        )).unwrap();
        self.refresh();
    }

    pub fn reset_card(&self, deck: String, id: i64) {
        let card = to_json(Card::new());
        Connection::open(Self::get_path()).unwrap().execute(format!("UPDATE \"{}\" SET DUE = '{}', STABILITY = {}, DIFFICULTY = {}, ELAPSED_DAYS = {}, SCHEDULED_DAYS = {}, REPS = {}, LAPSES = {}, STATE = {}, LAST_REVIEW = '{}', PREVIOUS_STATE = {}, LOG_RATING = {}, LOG_ELAPSED_DAYS = {}, LOG_SCHEDULED_DAYS = {}, LOG_STATE = {}, LOG_REVIEWED_DATE = '{}', FIRST_STUDY = NULL WHERE ROWID = {}",
            deck.replace(r#"""#, r#""""#),
            card["due"].as_str().unwrap().to_string(),
            card["stability"],
            card["difficulty"],
            card["elapsed_days"],
            card["scheduled_days"],
            card["reps"],
            card["lapses"],
            card["state"],
            card["last_review"].as_str().unwrap().to_string(),
            card["previous_state"],
            card["log"]["rating"],
            card["log"]["elapsed_days"],
            card["log"]["scheduled_days"],
            card["log"]["state"],
            card["log"]["reviewed_date"],
            id
        )).unwrap();
        self.refresh();
    }

    pub fn delete_card(&self, deck: String, id: i64) {
        Connection::open(Self::get_path()).unwrap().execute(format!("DELETE FROM \"{}\" WHERE ROWID = {}", deck.replace(r#"""#, r#""""#), id)).unwrap();
        self.refresh();
    }
}
