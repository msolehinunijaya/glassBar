//! SQLite storage for to-dos, notes, alerts, timers, screen time, caches.
//! Migrations only move forward; an existing file is never recreated.
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::Mutex;

pub struct Db(pub Mutex<Connection>);

/// Schema steps; PRAGMA user_version = how many have run.
pub const MIGRATIONS: &[&str] = &[r"
CREATE TABLE todos(id INTEGER PRIMARY KEY, text TEXT NOT NULL, done INT NOT NULL DEFAULT 0, due_at INT, created_at INT NOT NULL);
CREATE TABLE notes(id INTEGER PRIMARY KEY, text TEXT NOT NULL DEFAULT '', color TEXT NOT NULL DEFAULT 'yellow',
  pinned INT NOT NULL DEFAULT 0, images TEXT NOT NULL DEFAULT '[]', updated_at INT NOT NULL, deleted_at INT);
CREATE TABLE alerts(id INTEGER PRIMARY KEY, source TEXT NOT NULL, title TEXT NOT NULL, body TEXT NOT NULL,
  at INT NOT NULL, missed INT NOT NULL DEFAULT 0, read INT NOT NULL DEFAULT 0);
CREATE TABLE timers(id INTEGER PRIMARY KEY, label TEXT NOT NULL, duration_ms INT NOT NULL, ends_at INT, remaining_ms INT);
CREATE TABLE screen_time(day TEXT NOT NULL, exe TEXT NOT NULL, seconds INT NOT NULL, PRIMARY KEY(day, exe));
CREATE TABLE kv(key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE fired(key TEXT PRIMARY KEY, at INT NOT NULL);
", r"
CREATE TABLE ocr_history(id INTEGER PRIMARY KEY, at INT NOT NULL, text TEXT NOT NULL, mode TEXT NOT NULL, engine TEXT NOT NULL);
"];

pub fn migrate(c: &Connection) -> rusqlite::Result<()> {
    let v: usize = c.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))? as usize;
    for (i, sql) in MIGRATIONS.iter().enumerate().skip(v) {
        c.execute_batch(&format!("BEGIN; {sql}; PRAGMA user_version = {}; COMMIT;", i + 1))?;
    }
    Ok(())
}

pub fn open(path: &Path) -> rusqlite::Result<Db> {
    if let Some(dir) = path.parent() { let _ = std::fs::create_dir_all(dir); }
    let c = Connection::open(path)?;
    c.busy_timeout(std::time::Duration::from_secs(5))?;
    c.pragma_update(None, "journal_mode", "WAL")?;
    migrate(&c)?;
    Ok(Db(Mutex::new(c)))
}

pub fn kv_get(c: &Connection, key: &str) -> Option<String> {
    c.query_row("SELECT value FROM kv WHERE key = ?1", [key], |r| r.get(0)).optional().ok().flatten()
}

pub fn kv_set(c: &Connection, key: &str, value: &str) {
    let _ = c.execute("INSERT INTO kv(key, value) VALUES(?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value", params![key, value]);
}

pub fn now_ms() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("glassbar-db-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d.join("glassbar.db")
    }

    #[test]
    fn migrate_is_idempotent_and_keeps_rows() {
        let c = rusqlite::Connection::open_in_memory().unwrap();
        migrate(&c).unwrap();
        c.execute("INSERT INTO todos(text, created_at) VALUES ('a', 1)", []).unwrap();
        migrate(&c).unwrap();
        let n: i64 = c.query_row("SELECT count(*) FROM todos", [], |r| r.get(0)).unwrap();
        let v: i64 = c.query_row("PRAGMA user_version", [], |r| r.get(0)).unwrap();
        assert_eq!((n, v), (1, MIGRATIONS.len() as i64));
    }

    #[test]
    fn kv_round_trip() {
        let c = rusqlite::Connection::open_in_memory().unwrap();
        migrate(&c).unwrap();
        assert_eq!(kv_get(&c, "x"), None);
        kv_set(&c, "x", "1");
        kv_set(&c, "x", "2");
        assert_eq!(kv_get(&c, "x").as_deref(), Some("2"));
    }

    #[test]
    fn open_does_not_truncate_existing_file() {
        let p = tmp("reopen");
        {
            let db = open(&p).unwrap();
            db.0.lock().unwrap().execute("INSERT INTO todos(text, created_at) VALUES ('keep', 1)", []).unwrap();
        }
        let db = open(&p).unwrap();
        let t: String = db.0.lock().unwrap().query_row("SELECT text FROM todos", [], |r| r.get(0)).unwrap();
        assert_eq!(t, "keep");
    }
}
