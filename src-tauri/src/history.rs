use rusqlite::{Connection, Result};
use crate::types::HistoryEntry;
use std::path::PathBuf;

pub struct HistoryManager {
    db_path: PathBuf,
}

impl HistoryManager {
    pub fn new(app_dir: PathBuf) -> Result<Self> {
        let db_path = app_dir.join("history.db");
        let conn = Connection::open(&db_path)?;

        // Create table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                lcsc_id TEXT NOT NULL,
                component_name TEXT,
                success INTEGER NOT NULL,
                timestamp TEXT NOT NULL,
                output_dir TEXT NOT NULL
            )",
            [],
        )?;

        Ok(Self { db_path })
    }

    pub fn add_entry(&self, entry: &HistoryEntry) -> Result<()> {
        let conn = Connection::open(&self.db_path)?;
        conn.execute(
            "INSERT INTO history (lcsc_id, component_name, success, timestamp, output_dir)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                &entry.lcsc_id,
                &entry.component_name.as_deref().unwrap_or(""),
                if entry.success { 1 } else { 0 },
                &entry.timestamp,
                &entry.output_dir,
            ],
        )?;
        Ok(())
    }

    pub fn get_recent(&self, limit: usize) -> Result<Vec<HistoryEntry>> {
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare(
            "SELECT id, lcsc_id, component_name, success, timestamp, output_dir
             FROM history ORDER BY id DESC LIMIT ?1"
        )?;

        let entries = stmt.query_map([limit], |row| {
            let component_name_str: String = row.get(2)?;
            let component_name = if component_name_str.is_empty() {
                None
            } else {
                Some(component_name_str)
            };

            Ok(HistoryEntry {
                id: row.get(0)?,
                lcsc_id: row.get(1)?,
                component_name,
                success: row.get::<_, i64>(3)? != 0,
                timestamp: row.get(4)?,
                output_dir: row.get(5)?,
            })
        })?;

        entries.collect()
    }

    pub fn clear_all(&self) -> Result<()> {
        let conn = Connection::open(&self.db_path)?;
        conn.execute("DELETE FROM history", [])?;
        Ok(())
    }
}
