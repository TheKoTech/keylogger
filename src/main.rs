use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::BufReader;
use std::sync::Mutex;
use std::path::PathBuf;
use chrono::Local;
use rdev::{listen, Event, EventType, Key};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Default)]
struct DayStats {
    keys: HashMap<String, u32>,
}

fn main() {
    create_dir_all("stats").unwrap();

    let stats = Arc::new(Mutex::new(load_or_create_stats_for_today()));
    let pressed_keys = Arc::new(Mutex::new(HashSet::new()));

    println!("Started listening for keyboard events. Press Ctrl+C to exit.");

    if let Err(error) = listen(move |event| {
        handle_event(&event, Arc::clone(&stats), Arc::clone(&pressed_keys));
    }) {
        println!("Error: {:?}", error)
    }
}

fn get_stats_path(date: &str) -> PathBuf {
    PathBuf::from("stats").join(format!("{}.json", date))
}

fn load_or_create_stats_for_today() -> DayStats {
    let today = Local::now().format("%y.%m.%d").to_string();
    let path = get_stats_path(&today);

    match File::open(path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            serde_json::from_reader(reader).unwrap_or_default()
        }
        Err(_) => DayStats::default()
    }
}

fn save_stats(stats: &DayStats) {
    let today = Local::now().format("%y.%m.%d").to_string();
    let path = get_stats_path(&today);

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .unwrap();
    serde_json::to_writer_pretty(file, stats).unwrap();
}

fn key_to_string(key: Key) -> String {
    format!("{:?}", key)
}

fn handle_event(event: &Event, stats: Arc<Mutex<DayStats>>, pressed_keys: Arc<Mutex<HashSet<String>>>) {
    match event.event_type {
        EventType::KeyPress(key) => {
            let key_str = key_to_string(key);

            let mut pressed = pressed_keys.lock().unwrap();
            if pressed.contains(&key_str) {
                return;
            }
            pressed.insert(key_str.clone());

            let mut stats = stats.lock().unwrap();
            *stats.keys.entry(key_str).or_default() += 1;

            save_stats(&stats);
        }
        EventType::KeyRelease(key) => {
            let key_str = key_to_string(key);
            let mut pressed = pressed_keys.lock().unwrap();
            pressed.remove(&key_str);
        }
        _ => {}
    }
}
