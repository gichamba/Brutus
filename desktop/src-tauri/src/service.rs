//! Coordinate local file discovery, recovery workers and durable run snapshots.

use crate::{
    engine::{self, SearchResult},
    model::{FileResult, Run, Settings},
    pdf,
    store::Store,
};
use lopdf::Document;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    sync::{Arc, Mutex},
    thread,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

/// Owns one active run, its cancellation signal, and the local history store.
pub struct Service {
    /// Serialize access to the single SQLite history connection.
    store: Mutex<Store>,
    /// Shared run snapshot; workers update it before saving checkpoints.
    current: Mutex<Option<Run>>,
    /// Start instant and prior active milliseconds for the current session.
    timer: Mutex<Option<(Instant, u64)>>,
    /// Reserve the service until the coordinator joins every password worker.
    busy: AtomicBool,
    /// Stop discovery and candidate trials at their next cancellation check.
    cancel: AtomicBool,
}

/// Return the wall-clock time in milliseconds since the Unix epoch.
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

impl Service {
    /// Open history and create an idle service without launching workers.
    pub fn new(path: &Path) -> Result<Self, String> {
        Ok(Self {
            store: Mutex::new(Store::open(path)?),
            current: Mutex::new(None),
            timer: Mutex::new(None),
            busy: AtomicBool::new(false),
            cancel: AtomicBool::new(false),
        })
    }

    /// Read saved history under the store lock.
    pub fn history(&self) -> Result<Vec<Run>, String> {
        self.store
            .lock()
            .map_err(|_| "History lock failed")?
            .history()
    }

    /// Clone the current run and refresh elapsed time while it is active.
    pub fn snapshot(&self) -> Result<Option<Run>, String> {
        let mut run = self.current.lock().map_err(|_| "Run lock failed")?.clone();
        if let Some(run) = &mut run {
            if self.busy.load(Ordering::Acquire) {
                run.elapsed_ms = self.elapsed();
            }
        }
        Ok(run)
    }

    /// Accumulate active session milliseconds on top of prior resumed time.
    fn elapsed(&self) -> u64 {
        self.timer
            .lock()
            .ok()
            .and_then(|t| {
                t.as_ref()
                    .map(|(start, base)| base + start.elapsed().as_millis() as u64)
            })
            .unwrap_or(0)
    }

    /// Keep snapshot mutation and persistence in the same order across workers.
    fn update(&self, edit: impl FnOnce(&mut Run), save: bool) -> Result<(), String> {
        let mut current = self.current.lock().map_err(|_| "Run lock failed")?;
        let run = current.as_mut().ok_or("There is no active run")?;
        edit(run);
        run.elapsed_ms = self.elapsed();
        if save {
            let result = self
                .store
                .lock()
                .map_err(|_| "History lock failed")?
                .save(run);
            if let Err(error) = result {
                run.error = Some(format!(
                    "History could not be saved: {error}. Recovery stopped."
                ));
                run.status = "error".into();
                self.cancel.store(true, Ordering::Release);
                return Err(run.error.clone().unwrap_or_default());
            }
        }
        Ok(())
    }

    /// Require selected inputs and valid settings before reserving a run.
    pub fn start(
        self: &Arc<Self>,
        inputs: Vec<String>,
        settings: Settings,
    ) -> Result<String, String> {
        settings.validate()?;
        if inputs.is_empty() {
            return Err("Select a PDF file or folder first.".into());
        }
        let now = now_ms();
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let run = Run {
            id: format!(
                "{now}-{}-{}",
                std::process::id(),
                SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ),
            inputs,
            settings,
            status: "scanning".into(),
            started_at: now,
            finished_at: None,
            elapsed_ms: 0,
            files: vec![],
            warnings: vec![],
            error: None,
        };
        self.launch(run, false)
    }

    /// Restore a resumable run and let discovery revalidate its source content.
    pub fn resume(self: &Arc<Self>, id: &str) -> Result<String, String> {
        let mut run = self
            .store
            .lock()
            .map_err(|_| "History lock failed")?
            .get(id)?;
        if !run.resumable() {
            return Err("Only cancelled or interrupted runs can resume.".into());
        }
        run.status = "scanning".into();
        run.finished_at = None;
        run.error = None;
        self.launch(run, true)
    }

    /// Persist the initial snapshot and reserve one coordinator until it finishes.
    fn launch(self: &Arc<Self>, run: Run, resume: bool) -> Result<String, String> {
        if self
            .busy
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Err("A recovery run is already active.".into());
        }
        self.cancel.store(false, Ordering::Release);
        let id = run.id.clone();
        let setup = (|| {
            *self.timer.lock().map_err(|_| "Timer lock failed")? =
                Some((Instant::now(), run.elapsed_ms));
            self.store
                .lock()
                .map_err(|_| "History lock failed")?
                .save(&run)?;
            *self.current.lock().map_err(|_| "Run lock failed")? = Some(run);
            Ok::<_, String>(())
        })();
        if let Err(error) = setup {
            self.busy.store(false, Ordering::Release);
            return Err(error);
        }
        let service = self.clone();
        let spawn = thread::Builder::new()
            .name("brutus-run".into())
            .spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    service.execute(resume)
                }));
                let error = match result {
                    Ok(Ok(())) => None,
                    Ok(Err(error)) => Some(error),
                    Err(_) => Some("A recovery worker stopped unexpectedly.".into()),
                };
                let _ = service.update(
                    |run| {
                        if let Some(error) = error {
                            if run.error.is_none() {
                                run.error = Some(error);
                            }
                        }
                        run.status = if run.error.is_some() {
                            "error"
                        } else if service.cancel.load(Ordering::Acquire) {
                            "cancelled"
                        } else if run.files.iter().any(|f| f.status == "error")
                            || !run.warnings.is_empty()
                        {
                            "completed_with_errors"
                        } else {
                            "completed"
                        }
                        .into();
                        run.finished_at = Some(now_ms());
                        for file in &mut run.files {
                            if !file.terminal() {
                                file.status = "cancelled".into();
                            }
                        }
                    },
                    true,
                );
                service.busy.store(false, Ordering::Release);
            });
        if let Err(error) = spawn {
            let _ = self.update(
                |run| {
                    run.status = "interrupted".into();
                    run.error = Some(error.to_string());
                },
                true,
            );
            self.busy.store(false, Ordering::Release);
            return Err(error.to_string());
        }
        Ok(id)
    }

    /// Request cancellation without changing a run that has already finished.
    pub fn cancel(&self) -> Result<(), String> {
        if !self.busy.load(Ordering::Acquire) {
            return Ok(());
        }
        self.update(
            |run| {
                if matches!(run.status.as_str(), "scanning" | "running" | "cancelling") {
                    self.cancel.store(true, Ordering::Release);
                    run.status = "cancelling".into();
                }
            },
            false,
        )
    }

    /// Discover and prepare inputs, then join all workers before finalizing the run.
    fn execute(self: &Arc<Self>, resume: bool) -> Result<(), String> {
        let initial = self.snapshot()?.ok_or("No active run")?;
        let mut paths = BTreeSet::new();
        let mut warnings = vec![];
        for input in &initial.inputs {
            discover(Path::new(input), &mut paths, &mut warnings, &self.cancel);
        }
        let mut saved: BTreeMap<_, _> = if resume {
            initial
                .files
                .into_iter()
                .map(|f| (f.path.clone(), f))
                .collect()
        } else {
            BTreeMap::new()
        };
        let mut discovered: Vec<_> = paths
            .into_iter()
            .map(|path| saved.remove(&path).unwrap_or_else(|| FileResult::new(path)))
            .collect();
        discovered.extend(saved.into_values());
        self.update(
            |run| {
                run.files = discovered;
                run.warnings = warnings;
            },
            true,
        )?;
        if self.cancel.load(Ordering::Acquire) {
            return Ok(());
        }
        let files = self.snapshot()?.ok_or("No active run")?.files;
        if files.is_empty() {
            return Err("No readable PDF files were found in the selected inputs.".into());
        }
        let mut documents = Vec::with_capacity(files.len());
        for (index, file) in files.iter().enumerate() {
            if self.cancel.load(Ordering::Acquire) {
                return Ok(());
            }
            match self.prepare(file) {
                Ok((result, document)) => {
                    self.update(|run| run.files[index] = result, true)?;
                    documents.push(document.map(Arc::new));
                }
                Err(error) => {
                    self.update(
                        |run| {
                            let mut result = FileResult::new(file.path.clone());
                            result.status = "error".into();
                            result.error = Some(error);
                            run.files[index] = result;
                        },
                        true,
                    )?;
                    documents.push(None);
                }
            }
        }
        self.update(|run| run.status = "running".into(), true)?;
        let run = self.snapshot()?.ok_or("No active run")?;
        let completed: Vec<_> = run.files.iter().map(|f| f.completed.clone()).collect();
        let work = engine::jobs(&completed, run.settings.start, run.settings.end)?;
        let work = Arc::new(Mutex::new(work));
        let documents = Arc::new(documents);
        let finished: Arc<Vec<_>> = Arc::new(
            run.files
                .iter()
                .map(|f| AtomicBool::new(f.terminal()))
                .collect(),
        );
        let range_count = engine::ranges(run.settings.start, run.settings.end)?.len();
        let mut workers = vec![];
        let mut spawn_error = None;
        for worker in 0..run.settings.workers {
            let service = self.clone();
            let work = work.clone();
            let documents = documents.clone();
            let finished = finished.clone();
            let spawned = thread::Builder::new()
                .name(format!("brutus-worker-{worker}"))
                .spawn(move || -> Result<(), String> {
                    loop {
                        if service.cancel.load(Ordering::Acquire) {
                            break;
                        }
                        let job = work
                            .lock()
                            .map_err(|_| "Work queue lock failed")?
                            .pop_front();
                        let Some(job) = job else {
                            break;
                        };
                        if finished[job.file].load(Ordering::Relaxed) {
                            continue;
                        }
                        let Some(document) = &documents[job.file] else {
                            continue;
                        };
                        service.update(
                            |run| {
                                if !run.files[job.file].terminal() {
                                    run.files[job.file].status = "running".into();
                                }
                            },
                            false,
                        )?;
                        let result = engine::search_range(
                            job.range,
                            &service.cancel,
                            &finished[job.file],
                            |password| pdf::authenticate(document, password),
                            |attempts| {
                                let _ = service
                                    .update(|run| run.files[job.file].attempts += attempts, false);
                            },
                        );
                        service.update(
                            |run| {
                                let file = &mut run.files[job.file];
                                if file.terminal() {
                                    return;
                                }
                                match result {
                                    SearchResult::Found(password) => {
                                        file.status = "found".into();
                                        file.password = Some(password);
                                    }
                                    SearchResult::Exhausted => {
                                        file.completed.push(job.index);
                                        if file.completed.len() == range_count {
                                            file.status = "not_found".into();
                                            finished[job.file].store(true, Ordering::Relaxed);
                                        }
                                    }
                                    SearchResult::Error(error) => {
                                        file.status = "error".into();
                                        file.error = Some(error);
                                    }
                                    SearchResult::Interrupted => {}
                                }
                            },
                            true,
                        )?;
                    }
                    Ok(())
                });
            match spawned {
                Ok(worker) => workers.push(worker),
                Err(error) => {
                    self.cancel.store(true, Ordering::Release);
                    spawn_error = Some(error.to_string());
                    break;
                }
            }
        }
        let mut error = spawn_error;
        for worker in workers {
            match worker.join() {
                Ok(Ok(())) => {}
                Ok(Err(message)) => {
                    self.cancel.store(true, Ordering::Release);
                    error = Some(message);
                }
                Err(_) => {
                    self.cancel.store(true, Ordering::Release);
                    error = Some("A password worker panicked.".into());
                }
            }
        }
        if let Some(error) = error {
            return Err(error);
        }
        Ok(())
    }

    /// Check the reported 100 MiB size limit, read the source and revalidate content.
    /// Read failures return an error; parse failures return a file error outcome.
    fn prepare(&self, previous: &FileResult) -> Result<(FileResult, Option<Document>), String> {
        let path = Path::new(&previous.path);
        let metadata = fs::metadata(path).map_err(|e| format!("Cannot read PDF: {e}"))?;
        if metadata.len() > 100 * 1024 * 1024 {
            return Err("This PDF exceeds the 100 MB file limit.".into());
        }
        let bytes = fs::read(path).map_err(|e| format!("Cannot read PDF: {e}"))?;
        Ok(prepare_content(previous, &bytes, |hash| {
            self.store
                .lock()
                .map_err(|_| "History lock failed")?
                .cached(hash)
        }))
    }
}

/// Preserve content resets even when parsing or password authentication fails.
fn prepare_content(
    previous: &FileResult,
    bytes: &[u8],
    cached: impl FnOnce(&str) -> Result<Option<Option<String>>, String>,
) -> (FileResult, Option<Document>) {
    let hash = format!("{:x}", Sha256::digest(bytes));
    let mut file = previous.clone();
    file.reset_if_changed(hash);
    let result = (|| {
        if file.terminal() {
            return Ok(None);
        }
        if let Some(password) = cached(&file.hash)? {
            file.status = if password.is_some() {
                "found"
            } else {
                "unprotected"
            }
            .into();
            file.password = password;
            file.reused = true;
            return Ok(None);
        }
        let document = Document::load_mem(bytes).map_err(|e| format!("Cannot parse PDF: {e}"))?;
        if !pdf::protected(&document)? {
            file.status = "unprotected".into();
            return Ok(None);
        }
        file.status = "queued".into();
        file.error = None;
        Ok(Some(document))
    })();
    match result {
        Ok(document) => (file, document),
        Err(error) => {
            file.status = "error".into();
            file.error = Some(error);
            file.password = None;
            file.reused = false;
            (file, None)
        }
    }
}

#[cfg(test)]
/// Pure preparation regression checks that do not open files or databases.
mod tests {
    use super::*;

    #[test]
    /// Malformed replacement bytes must not retain an earlier password or work.
    fn malformed_replacement_discards_old_password_and_checkpoints() {
        let mut previous = FileResult::new("statement.pdf".into());
        previous.hash = "old-content".into();
        previous.status = "found".into();
        previous.password = Some("123456".into());
        previous.completed = vec![0, 1];
        previous.attempts = 2000;
        let (file, document) = prepare_content(&previous, b"This is not a PDF", |_| Ok(None));
        assert_eq!(file.status, "error");
        assert!(file.error.is_some());
        assert!(document.is_none());
        assert_eq!(file.password, None);
        assert!(file.completed.is_empty());
        assert_eq!(file.attempts, 0);
        assert_ne!(file.hash, previous.hash);
    }
}

/// Ignore directory symlinks to avoid traversal cycles; report unreadable paths.
fn discover(
    path: &Path,
    found: &mut BTreeSet<String>,
    warnings: &mut Vec<String>,
    cancel: &AtomicBool,
) {
    if cancel.load(Ordering::Acquire) {
        return;
    }
    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        Err(e) => {
            warnings.push(format!("{}: {e}", path.display()));
            return;
        }
    };
    if metadata.is_file() {
        if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("pdf"))
        {
            match fs::canonicalize(path) {
                Ok(p) => {
                    found.insert(p.to_string_lossy().into_owned());
                }
                Err(e) => warnings.push(format!("{}: {e}", path.display())),
            }
        } else {
            warnings.push(format!("{} is not a PDF file.", path.display()));
        }
    } else if metadata.is_dir() {
        let entries = match fs::read_dir(path) {
            Ok(e) => e,
            Err(e) => {
                warnings.push(format!("{}: {e}", path.display()));
                return;
            }
        };
        for entry in entries {
            if cancel.load(Ordering::Acquire) {
                break;
            }
            match entry {
                Ok(entry) => match entry.file_type() {
                    Ok(kind)
                        if kind.is_dir()
                            || (kind.is_file()
                                && entry
                                    .path()
                                    .extension()
                                    .is_some_and(|e| e.eq_ignore_ascii_case("pdf"))) =>
                    {
                        discover(&entry.path(), found, warnings, cancel)
                    }
                    Ok(_) => {}
                    Err(e) => warnings.push(format!("{}: {e}", entry.path().display())),
                },
                Err(e) => warnings.push(format!("{}: {e}", path.display())),
            }
        }
    } else {
        warnings.push(format!("{} is not a file or folder.", path.display()));
    }
}

/// Resolve the user-specific history directory during Tauri setup.
pub fn history_path(directory: PathBuf) -> Result<PathBuf, String> {
    fs::create_dir_all(&directory).map_err(|e| format!("Cannot create history directory: {e}"))?;
    Ok(directory.join("brutus-desktop.db"))
}
