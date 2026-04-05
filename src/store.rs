use std::{
    collections::HashMap,
    fs,
    io::{BufRead, BufReader, Write},
    sync::{Arc, RwLock},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use serde::{Deserialize, Serialize};

pub struct Entry {
    pub value: String,
    pub expires_at: Option<Instant>,
}

impl Entry {
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(deadline) => Instant::now() > deadline,
            None => false,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct DiskEntry {
    pub key: String,
    pub value: String,
    pub expires_at: Option<u64>,  
}

pub type Db = Arc<RwLock<HashMap<String, Entry>>>;

pub fn new_db() -> Db {
    Arc::new(RwLock::new(HashMap::new()))
}

fn unix_to_instant(unix_secs: u64) -> Option<Instant> {
    let now_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if unix_secs <= now_unix {
        None 
    } else {
        let secs_remaining = unix_secs - now_unix;
        Some(Instant::now() + Duration::from_secs(secs_remaining))
    }
}

fn instant_to_unix(instant: Instant) -> u64 {
    let now_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let secs_remaining = instant
        .checked_duration_since(Instant::now())
        .unwrap_or_default()  // if expired, treat as 0 remaining
        .as_secs();

    now_unix + secs_remaining
}

pub fn save(db: &Db, path: &str) -> std::io::Result<()> {
    let lock = db.read().unwrap();
    let mut file = fs::File::create(path)?;

    for (key, entry) in lock.iter() {
        if entry.is_expired() {
            continue;  
        }

        let disk_entry = DiskEntry {
            key: key.clone(),
            value: entry.value.clone(),
            expires_at: entry.expires_at.map(instant_to_unix),
        };

        let line = serde_json::to_string(&disk_entry).unwrap();
        writeln!(file, "{}", line)?;
    }

    Ok(())
}

pub fn load(db: &Db, path: &str) -> std::io::Result<()> {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return Ok(()),  
    };

    let mut lock = db.write().unwrap();

    for line in BufReader::new(file).lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }

        let disk_entry: DiskEntry = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Skipping corrupt line: {}", e);
                continue;
            }
        };

        let expires_at = match disk_entry.expires_at {
            None => None,
            Some(unix_secs) => match unix_to_instant(unix_secs) {
                None => continue, 
                Some(instant) => Some(instant),
            },
        };

        lock.insert(disk_entry.key, Entry {
            value: disk_entry.value,
            expires_at,
        });
    }

    Ok(())
}
