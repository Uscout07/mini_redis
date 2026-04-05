# mini-redis

A lightweight, Redis-inspired key-value store built in Rust. Handles concurrent TCP clients, key expiry with TTL, and persistence across restarts.

Built as a learning project to explore async Rust, shared state across threads, and systems programming patterns.

---

## Features

- **Concurrent clients** — async TCP server using `tokio`, each client gets its own task
- **Core commands** — `SET`, `GET`, `DELETE`, `KEYS`
- **TTL support** — `SETEX` sets a key with an expiry; `TTL` returns seconds remaining
- **Lazy + active expiry** — expired keys are hidden on read and cleaned up by a background task every second
- **Persistence** — data survives restarts via newline-delimited JSON; manual `SAVE` command and auto-save every 30 seconds
- **Graceful shutdown** — Ctrl+C triggers a final save before exit

---

## Quick Start

**Requirements:** Rust + Cargo ([install here](https://rustup.rs))

```bash
git clone https://github.com/yourname/mini-redis
cd mini-redis
cargo run
```

In another terminal:

```bash
nc 127.0.0.1 6379
```

---

## Commands

| Command | Example | Response |
|---|---|---|
| `SET key value` | `SET name alice` | `OK` |
| `GET key` | `GET name` | `alice` |
| `DELETE key` | `DELETE name` | `OK` |
| `SETEX key seconds value` | `SETEX temp 30 hello` | `OK` |
| `TTL key` | `TTL temp` | `28` |
| `KEYS` | `KEYS` | all live keys, one per line |
| `SAVE` | `SAVE` | `OK` |

**TTL return values:**
- Positive integer — seconds remaining
- `-1` — key exists but has no expiry
- `-2` — key does not exist
- `0` — key is expired

---

## Architecture

```
src/
├── main.rs       # boots the server, spawns background tasks, handles shutdown
├── store.rs      # HashMap wrapped in Arc<RwLock<T>>, save/load logic
├── command.rs    # parses raw TCP input into typed Command enum
└── handler.rs    # handles one client connection end-to-end
```

### Key design decisions

**`Arc<RwLock<HashMap<String, Entry>>>`** — multiple clients can read concurrently; writes get exclusive access. `Arc` lets every task share ownership without any single one holding it.

**Two-struct pattern** — `Entry` uses `Instant` for fast in-memory expiry checks. `DiskEntry` uses Unix timestamps (`u64`) for serialization. `Instant` is an opaque OS timer and can't cross a process boundary — the conversion layer handles this on save and load.

**Lazy + active expiry** — reads check expiry inline (lazy). A background task calls `HashMap::retain` every second to free memory (active). Same approach used by real Redis.

---

## Persistence Format

Data is stored in `mini_redis.db` as newline-delimited JSON:

```json
{"key":"name","value":"alice","expires_at":null}
{"key":"temp","value":"hello","expires_at":1712345678}
```

On startup, entries with an `expires_at` in the past are silently skipped. Corrupt lines are logged and skipped without crashing.

---

## Dependencies

```toml
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

---

## What I Learned

This project was built to get hands-on with:

- `async/await` and `tokio::spawn` for concurrent task management
- `Arc<RwLock<T>>` for shared mutable state across threads
- Rust's ownership model under real pressure (multiple tasks, one store)
- `Instant` vs `SystemTime` and why they're different
- `serde` derive macros for zero-boilerplate serialization
- `tokio::signal` for graceful shutdown
- Match guards, iterator chaining, and exhaustive enum matching

---

## Possible Next Steps

- `KEYS pattern` — glob pattern filtering
- `INCR` / `DECR` — atomic integer operations  
- `EXPIRE key seconds` — add TTL to an existing key
- Proper client protocol (RESP) instead of plain text
- Integration tests with `tokio::test`
- Replace `.unwrap()` with structured errors using `thiserror`
