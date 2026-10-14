//! Serializable settings, file checkpoints and run history records.

use serde::{Deserialize, Serialize};

/// Settings recorded with each recovery run.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// Inclusive first numeric candidate, from 0 to 999999.
    pub start: u32,
    /// Inclusive last numeric candidate, from start to 999999.
    pub end: u32,
    /// Maximum simultaneous password workers, from 1 to 32.
    pub workers: usize,
}

impl Settings {
    /// Reject out-of-scope numeric ranges and worker limits before launching.
    pub fn validate(&self) -> Result<(), String> {
        crate::engine::ranges(self.start, self.end)?;
        if !(1..=32).contains(&self.workers) {
            return Err("Choose between 1 and 32 workers.".into());
        }
        Ok(())
    }
}

/// A file's outcome and durable completed-range checkpoints.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileResult {
    /// Local source path displayed in the run and history.
    pub path: String,
    /// SHA256 of searched bytes; empty when those bytes could not be read.
    pub hash: String,
    /// Current queued, running, cancelled or terminal outcome label.
    pub status: String,
    /// Recovered user password, present only for a successful recovery.
    pub password: Option<String>,
    /// File-specific failure message, when processing failed.
    pub error: Option<String>,
    /// Actual candidate trials, including partial ranges retried on resume.
    pub attempts: u64,
    /// Fully searched range indexes for the recorded run settings.
    pub completed: Vec<u32>,
    /// Whether the result came from the successful content-hash cache.
    pub reused: bool,
}

impl FileResult {
    /// Create a queued file without a hash, password or checkpoints.
    pub fn new(path: String) -> Self {
        Self {
            path,
            hash: String::new(),
            status: "queued".into(),
            password: None,
            error: None,
            attempts: 0,
            completed: vec![],
            reused: false,
        }
    }

    /// Whether this file has an outcome that stops scheduling further jobs.
    pub fn terminal(&self) -> bool {
        matches!(
            self.status.as_str(),
            "found" | "unprotected" | "not_found" | "error"
        )
    }

    /// Checkpoints apply only to the exact bytes that were searched.
    pub fn reset_if_changed(&mut self, hash: String) {
        if self.hash != hash {
            *self = Self::new(self.path.clone());
            self.hash = hash;
        }
    }
}

/// A persisted run; timestamps and elapsed durations are milliseconds.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run {
    /// Local identifier used to save and resume this run.
    pub id: String,
    /// Selected file or folder paths, retained for discovery on resume.
    pub inputs: Vec<String>,
    /// Numeric range and worker limit used throughout the run.
    pub settings: Settings,
    /// Current run state, including completion, cancellation and errors.
    pub status: String,
    /// Initial start time in milliseconds since the Unix epoch.
    pub started_at: u64,
    /// Latest finish time in milliseconds since the Unix epoch, if finished.
    pub finished_at: Option<u64>,
    /// Accumulated active processing time in milliseconds across resumes.
    pub elapsed_ms: u64,
    /// Discovered files and their individual outcomes and checkpoints.
    pub files: Vec<FileResult>,
    /// Discovery warnings for skipped or unreadable inputs.
    pub warnings: Vec<String>,
    /// Run-level failure, including an inability to save history.
    pub error: Option<String>,
}

impl Run {
    /// Only cancelled and interrupted runs may resume.
    pub fn resumable(&self) -> bool {
        matches!(self.status.as_str(), "cancelled" | "interrupted")
    }
}

#[cfg(test)]
/// Check content-bound checkpoint resets and settings validation.
mod tests {
    use super::*;

    #[test]
    /// Replacement bytes invalidate the password, trials and completed ranges.
    fn changed_content_clears_password_and_checkpoints() {
        let mut file = FileResult::new("invoice.pdf".into());
        file.hash = "old".into();
        file.completed = vec![0, 1];
        file.attempts = 2000;
        file.password = Some("123456".into());
        file.status = "found".into();
        file.reset_if_changed("new".into());
        assert!(file.completed.is_empty());
        assert_eq!(file.attempts, 0);
        assert_eq!(file.password, None);
        assert_eq!(file.status, "queued");
    }

    #[test]
    /// The same content hash retains its completed range indexes.
    fn unchanged_content_preserves_checkpoint() {
        let mut file = FileResult::new("invoice.pdf".into());
        file.hash = "same".into();
        file.completed = vec![7];
        file.reset_if_changed("same".into());
        assert_eq!(file.completed, vec![7]);
    }

    #[test]
    /// Zero and 33 workers are invalid; four workers are allowed.
    fn worker_limit_is_validated() {
        assert!(Settings {
            start: 0,
            end: 999999,
            workers: 0
        }
        .validate()
        .is_err());
        assert!(Settings {
            start: 0,
            end: 999999,
            workers: 33
        }
        .validate()
        .is_err());
        assert!(Settings {
            start: 0,
            end: 999999,
            workers: 4
        }
        .validate()
        .is_ok());
    }
}
