use crate::users::Storage;
use std::fs::create_dir_all;
use std::io::prelude::*;
use std::{collections::HashMap, path::PathBuf};

pub struct Database {
    map: HashMap<String, String>,
    path: PathBuf,
}

/// Implementation of the Database struct
impl Database {
    pub fn new(path: &PathBuf) -> Result<Database, std::io::Error> {
        let mut map: HashMap<String, String> = HashMap::new();
        let mut contents = String::new();

        if path.exists() {
            let file = std::fs::File::open(path)?;
            let mut buf_reader = std::io::BufReader::new(file);
            buf_reader.read_to_string(&mut contents)?;
        } else {
            std::fs::File::create(path)?;
        }
        for line in contents.lines() {
            let (key, value) = line.split_once("\t").expect("Corrupt database");
            map.insert(key.to_string(), value.to_string());
        }
        let path = path.clone();
        Ok(Database { map, path })
    }
    pub fn remove(&mut self, key: &String) {
        self.map.remove(key);
    }

    pub fn list(&self) -> HashMap<String, String> {
        let mut data = self.map.clone();
        data.remove(&"auth_token".to_string());
        data.remove(&"list_uuid".to_string());
        data.remove(&"expiration_timestamp".to_string());
        data
    }
}

impl Storage for Database {
    fn insert(&mut self, key: String, value: String) {
        self.map.insert(key, value);
    }

    fn get(&self, key: &str) -> Option<&String> {
        self.map.get(key)
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        let _ = do_flush(self);
    }
}

fn do_flush(database: &Database) -> std::io::Result<()> {
    let mut contents = String::new();
    for (key, value) in &database.map {
        contents.push_str(key);
        contents.push('\t');
        contents.push_str(value);
        contents.push('\n');
    }
    std::fs::write(database.path.clone(), contents)
}

pub fn create_database(path: &mut PathBuf) -> Result<Database, std::io::Error> {
    create_dir_all(path.clone())?;
    path.push("kv.db");
    Database::new(&path)
}
