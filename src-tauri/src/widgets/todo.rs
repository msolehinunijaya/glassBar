//! To-do list (one shared list). Due items feed the scheduler.
use crate::db::{now_ms, Db};
use crate::scheduler::{self, Due};
use rusqlite::params;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Todo { pub id: i64, pub text: String, pub done: bool, pub due_at: Option<i64>, pub created_at: i64 }

pub fn due_items(app: &AppHandle) -> Vec<Due> {
    let db = app.state::<Db>();
    let c = db.0.lock().unwrap();
    let mut q = c.prepare("SELECT id, text, due_at FROM todos WHERE done = 0 AND due_at IS NOT NULL").unwrap();
    q.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?)))
        .map(|rows| rows.filter_map(Result::ok)
            // The due time is part of the key, so changing it re-arms the alert.
            .map(|(id, text, at)| Due { key: format!("todo:{id}:{at}"), at, source: "todo", title: "To-do due".into(), body: text, sound: None })
            .collect())
        .unwrap_or_default()
}

fn changed(app: &AppHandle) {
    let _ = app.emit("todos:changed", ());
    scheduler::poke();
}

#[tauri::command]
pub fn todo_list(db: State<Db>) -> Vec<Todo> {
    let c = db.0.lock().unwrap();
    let mut q = c.prepare("SELECT id, text, done, due_at, created_at FROM todos ORDER BY done, COALESCE(due_at, 9e18), id DESC").unwrap();
    q.query_map([], |r| Ok(Todo { id: r.get(0)?, text: r.get(1)?, done: r.get(2)?, due_at: r.get(3)?, created_at: r.get(4)? }))
        .map(|rows| rows.filter_map(Result::ok).collect())
        .unwrap_or_default()
}

#[tauri::command]
pub fn todo_add(app: AppHandle, text: String, due_at: Option<i64>) -> Result<(), String> {
    let text = text.trim();
    if text.is_empty() { return Err("empty to-do".into()); }
    app.state::<Db>().0.lock().unwrap()
        .execute("INSERT INTO todos(text, due_at, created_at) VALUES(?1, ?2, ?3)", params![text, due_at, now_ms()])
        .map_err(|e| e.to_string())?;
    changed(&app);
    Ok(())
}

#[tauri::command]
pub fn todo_update(app: AppHandle, id: i64, text: String, done: bool, due_at: Option<i64>) -> Result<(), String> {
    app.state::<Db>().0.lock().unwrap()
        .execute("UPDATE todos SET text = ?2, done = ?3, due_at = ?4 WHERE id = ?1", params![id, text.trim(), done, due_at])
        .map_err(|e| e.to_string())?;
    changed(&app);
    Ok(())
}

#[tauri::command]
pub fn todo_delete(app: AppHandle, id: i64) {
    let _ = app.state::<Db>().0.lock().unwrap().execute("DELETE FROM todos WHERE id = ?1", [id]);
    changed(&app);
}

#[tauri::command]
pub fn todo_clear_done(app: AppHandle) {
    let _ = app.state::<Db>().0.lock().unwrap().execute("DELETE FROM todos WHERE done = 1", []);
    changed(&app);
}
