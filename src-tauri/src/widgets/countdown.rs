//! Countdowns: a notification on the day (at its time, or 9:00) and, if asked, at 9:00 the day
//! before. The countdowns live in the widget's settings; the dock keeps repeating ones moved
//! to their next date, so this only reads `date`.
use crate::scheduler::Due;
use chrono::{Local, NaiveDate, NaiveTime, TimeZone};
use tauri::{AppHandle, Manager};

/// Local wall time -> Unix ms (None for a time that doesn't exist, e.g. a DST gap).
fn local_ms(date: NaiveDate, time: NaiveTime) -> Option<i64> {
    Local.from_local_datetime(&date.and_time(time)).earliest().map(|t| t.timestamp_millis())
}

/// Pure: the due items for one countdown.
pub fn items_for(c: &serde_json::Value) -> Vec<Due> {
    let (Some(id), Some(name), Some(date)) = (c["id"].as_str(), c["name"].as_str(), c["date"].as_str()) else { return vec![] };
    let Ok(day) = NaiveDate::parse_from_str(date, "%Y-%m-%d") else { return vec![] };
    let nine = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
    let time = c["time"].as_str().and_then(|t| NaiveTime::parse_from_str(t, "%H:%M").ok());
    let emoji = c["emoji"].as_str().unwrap_or("");
    let mut v = vec![];
    if let Some(at) = local_ms(day, time.unwrap_or(nine)) {
        let body = match time { Some(t) => format!("{emoji} It's {}.", t.format("%H:%M")), None => format!("{emoji} The day is here.") };
        v.push(Due { key: format!("countdown:{id}:{date}"), at, source: "countdown", title: format!("{name} is today"), body: body.trim().into(), sound: None });
    }
    if c["remindDayBefore"] == true {
        if let Some(at) = day.pred_opt().and_then(|d| local_ms(d, nine)) {
            v.push(Due { key: format!("countdown:{id}:{date}:eve"), at, source: "countdown", title: format!("{name} is tomorrow"),
                body: format!("{emoji} {}", day.format("%A, %-d %B")).trim().into(), sound: None });
        }
    }
    v
}

pub fn due_items(app: &AppHandle) -> Vec<Due> {
    let cfg = app.state::<crate::AppState>().config.lock().unwrap().clone();
    if !cfg.uses("countdown") { return vec![]; }
    let s = cfg.widget_settings("countdown");
    if s["notify"] == false { return vec![]; }
    s["items"].as_array().map(|a| a.iter().flat_map(items_for).collect()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn on_the_day_at_nine_and_the_day_before() {
        let c = serde_json::json!({ "id": "x", "name": "Merdeka", "date": "2026-11-02", "time": null, "emoji": "🇲🇾", "remindDayBefore": true });
        let v = items_for(&c);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].key, "countdown:x:2026-11-02");
        assert_eq!(v[0].title, "Merdeka is today");
        let nine = Local.with_ymd_and_hms(2026, 11, 2, 9, 0, 0).unwrap().timestamp_millis();
        assert_eq!(v[0].at, nine);
        assert_eq!(v[1].key, "countdown:x:2026-11-02:eve");
        assert_eq!(v[1].at, nine - 86_400_000);
    }
    #[test]
    fn at_its_own_time_and_skips_bad_dates() {
        let c = serde_json::json!({ "id": "y", "name": "Launch", "date": "2026-11-02", "time": "18:30", "emoji": "", "remindDayBefore": false });
        let v = items_for(&c);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].at, Local.with_ymd_and_hms(2026, 11, 2, 18, 30, 0).unwrap().timestamp_millis());
        assert!(items_for(&serde_json::json!({ "id": "z", "name": "Bad", "date": "soon" })).is_empty());
    }
}
