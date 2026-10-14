//! Store local run snapshots and successful results in SQLite.

use crate::model::Run;
use rusqlite::{params, Connection};
use std::{path::Path, time::Duration};

/// All database writes are serialized by the application's store mutex.
pub struct Store {
    /// The connection used for serialized transactions and history reads.
    connection: Connection,
}

impl Store {
    /// Open the schema and mark saved active runs interrupted after a restart.
    pub fn open(path: &Path) -> Result<Self, String> {
        let connection = Connection::open(path).map_err(|e| e.to_string())?;
        connection
            .busy_timeout(Duration::from_secs(5))
            .map_err(|e| e.to_string())?;
        connection.execute_batch("PRAGMA journal_mode=WAL;
            CREATE TABLE IF NOT EXISTS runs (id TEXT PRIMARY KEY, started_at INTEGER NOT NULL, data TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS results (hash TEXT PRIMARY KEY, password TEXT);").map_err(|e| e.to_string())?;
        let mut store = Self { connection };
        for mut run in store.history()? {
            if matches!(run.status.as_str(), "running" | "scanning" | "cancelling") {
                run.status = "interrupted".into();
                for file in &mut run.files {
                    if !file.terminal() {
                        file.status = "queued".into();
                    }
                }
                store.save(&run)?;
            }
        }
        Ok(store)
    }

    /// Save the run and successful content-hash results in one transaction.
    pub fn save(&mut self, run: &Run) -> Result<(), String> {
        let data = serde_json::to_string(run).map_err(|e| e.to_string())?;
        let transaction = self.connection.transaction().map_err(|e| e.to_string())?;
        transaction.execute("INSERT INTO runs VALUES (?1, ?2, ?3) ON CONFLICT(id) DO UPDATE SET data=excluded.data",
            params![run.id, run.started_at, data]).map_err(|e| e.to_string())?;
        for file in &run.files {
            if !file.hash.is_empty() && matches!(file.status.as_str(), "found" | "unprotected") {
                transaction.execute("INSERT INTO results VALUES (?1, ?2) ON CONFLICT(hash) DO UPDATE SET password = ?2",
                    params![file.hash, file.password]).map_err(|e| e.to_string())?;
            }
        }
        transaction.commit().map_err(|e| e.to_string())
    }

    /// Read all saved runs, newest first, reporting malformed history records.
    pub fn history(&self) -> Result<Vec<Run>, String> {
        let mut query = self
            .connection
            .prepare("SELECT data FROM runs ORDER BY started_at DESC")
            .map_err(|e| e.to_string())?;
        let rows = query
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.map(|row| {
            let data = row.map_err(|e| e.to_string())?;
            serde_json::from_str(&data).map_err(|e| format!("Cannot read saved history: {e}"))
        })
        .collect()
    }

    /// Read the run with this ID, failing if it is missing or malformed.
    pub fn get(&self, id: &str) -> Result<Run, String> {
        let data: String = self
            .connection
            .query_row("SELECT data FROM runs WHERE id=?1", [id], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        serde_json::from_str(&data).map_err(|e| e.to_string())
    }

    /// Outer None means no cache entry; inner None means no password was needed.
    pub fn cached(&self, hash: &str) -> Result<Option<Option<String>>, String> {
        use rusqlite::OptionalExtension;
        self.connection
            .query_row(
                "SELECT password FROM results WHERE hash=?1",
                [hash],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())
    }
}
