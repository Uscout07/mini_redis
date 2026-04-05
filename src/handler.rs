use crate::store;
use std::time::{Duration, Instant};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

use crate::{
    command::Command,
    store::{Db, Entry},
};

pub async fn handle_client(stream: TcpStream, db: Db, path: String){
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();

        let bytes_read = reader.read_line(&mut line).await.unwrap_or(0);
        if bytes_read == 0 {
            break;
        }

        let response = match Command::parse(&line) {
            Command::Set { key, value } => {
                match db.write() {
                    Ok(mut lock) => {
                        lock.insert(key, Entry { value, expires_at: None });
                        "OK\n".to_string()
                    }
                    Err(e) => {
                        eprintln!("DB write error: {}", e);
                        "ERR internal server error\n".to_string()
                    }
                }
            }

            Command::SetEx { key, value, secs } => {
                match db.write() {
                    Ok(mut lock) => {
                        lock.insert(key, Entry {
                            value,
                            expires_at: Some(Instant::now() + Duration::from_secs(secs)),
                        });
                        "OK\n".to_string()
                    }
                    Err(e) => {
                        eprintln!("DB write error: {}", e);
                        "ERR internal server error\n".to_string()
                    }
                }
            }

            Command::Get { key } => match db.read() {
                Ok(lock) => match lock.get(&key) {
                    Some(entry) if entry.is_expired() => "NIL\n".to_string(),
                    Some(entry) => format!("{}\n", entry.value),
                    None => "NIL\n".to_string(),
                },
                Err(e) => {
                    eprintln!("DB read error: {}", e);
                    "ERR internal server error\n".to_string()
                }
            },

            Command::Delete { key } => match db.write() {
                Ok(mut lock) => {
                    lock.remove(&key);
                    "OK\n".to_string()
                }
                Err(e) => {
                    eprintln!("DB write error: {}", e);
                    "ERR internal server error\n".to_string()
                }
            },

            Command::Ttl {key} => match db.read() {
                Ok(lock) => match lock.get(&key){
                    None => "-2\n".to_string(), 
                    Some(entry) => match entry.expires_at {
                        None => "-1\n".to_string(),
                        Some(deadline) => match deadline.checked_duration_since(Instant::now()) {
                            Some(remaining) => format!("{}\n", remaining.as_secs()),
                            None => "0\n".to_string()
                        }
                    }
                },
                Err(e) => {
                    eprintln!("DB read error: {}", e);
                    "ERR internal server error\n".to_string()
                }

            }

            Command::Keys => match db.read() {
                Ok(lock) => {
                    let keys:Vec<&str> = lock.iter()
                        .filter(|(_k, v)| !v.is_expired())
                        .map(|(k, _v)| k.as_str())
                        .collect();

                    if keys.is_empty(){
                        "empty\n".to_string()
                    } else {
                        format!("{}\n", keys.join("\n"))
                    }
                }
                Err(e) => {
                    eprint!("DB read error: {}", e);
                    "ERR internal server error\n".to_string()
                }
            }

            Command::Save => match store::save(&db, &path) {
                    Ok(_) => "OK\n".to_string(),
                    Err(e) => {
                    eprintln!("Save error: {}", e);
                    "ERR save failed\n".to_string()
                    }
            },

            Command::Unknown(raw) => format!("ERR unknown command: {}\n", raw),
        };

        if writer.write_all(response.as_bytes()).await.is_err() {
            break;
        }
    }
}
