mod command;
mod handler;
mod store;

use std::time::Duration;
use tokio::{net::TcpListener, time, signal};

const SAVE_PATH: &str = "mini_redis.db";

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    println!("Listening on port 6379...");

    let db = store::new_db();

    if let Err(e) = store::load(&db, SAVE_PATH) {
        eprintln!("Failed to load save file: {}", e);
    } else {
        println!("Loaded data from {}", SAVE_PATH);
    }

    let db_autosave = db.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(30));
        interval.tick().await; // skip the immediate first tick
        loop {
            interval.tick().await;
            if let Err(e) = store::save(&db_autosave, SAVE_PATH) {
                eprintln!("Autosave failed: {}", e);
            } else {
                println!("Autosaved to {}", SAVE_PATH);
            }
        }
    });

    let db_shutdown = db.clone();
    tokio::spawn(async move {
        signal::ctrl_c().await.unwrap();
        println!("Shutting down, saving...");
        if let Err(e) = store::save(&db_shutdown, SAVE_PATH) {
            eprintln!("Failed to save on shutdown: {}", e);
        } else {
            println!("Saved to {}", SAVE_PATH);
        }
        std::process::exit(0);
    });

    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        println!("New connection: {}", addr);

        let db_clone = db.clone();
        tokio::spawn(async move {
            handler::handle_client(stream, db_clone, SAVE_PATH.to_string()).await;
        });
    }
}
