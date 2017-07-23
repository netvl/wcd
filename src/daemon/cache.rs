#![allow(dead_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::sync::Mutex;

use appdirs;
use bincode::{deserialize_from, Infinite};

const APP: &'static str = "wcd";

lazy_static! {
    static ref CACHE: Mutex<HashMap<String, String>> = {
        let cache_file = get_cache_file();
        let map = if cache_file.exists() {
            read_cache_file(&cache_file)
        } else {
            HashMap::new()
        };
        Mutex::new(map)
    };
}

fn read_cache_file(p: &Path) -> HashMap<String, String> {
    let mut f = File::open(p)
        .expect(&format!(
            "Cannot open cache file {}; check if it has valid permissions \
            and remove it if the problem persists",
            p.display()
        ));
    deserialize_from(&mut f, Infinite)
        .expect(&format!(
            "Failed to deserialize cache data from file {}; \
            remove it if the problem persists",
            p.display()
        ))
}

fn get_cache_file() -> PathBuf {
    let mut cache_dir = appdirs::user_cache_dir(Some(APP), None)
        .expect("Cannot obtain cache directory");
    cache_dir.push("cache.bincode");
    cache_dir
}

pub fn get(key: &str) -> Option<String> {
    let guard = CACHE.lock().expect("Failed to lock the cache mutex");
    guard.get(key).cloned()
}
