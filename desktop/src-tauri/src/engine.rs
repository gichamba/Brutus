//! Split numeric searches into jobs and run cancellable password trials.

/// An inclusive interval within the six-digit numeric password space.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range {
    /// First candidate, including leading zeroes when formatted for testing.
    pub start: u32,
    /// Last candidate to test, included in the interval.
    pub end: u32,
}

/// Split a valid inclusive interval into ranges of at most 1,000 candidates.
pub fn ranges(start: u32, end: u32) -> Result<Vec<Range>, String> {
    if start > end || end > 999_999 {
        return Err("Use an inclusive range between 000000 and 999999.".into());
    }
    Ok((start..=end)
        .step_by(1000)
        .map(|from| Range {
            start: from,
            end: (from + 999).min(end),
        })
        .collect())
}

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};

/// One file's range, with a stable checkpoint index for the run's settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Job {
    /// Position of the file in the run's result list.
    pub file: usize,
    /// Zero-based range index saved after the entire range completes.
    pub index: u32,
    /// Candidate interval assigned to this job.
    pub range: Range,
}

/// Interleave unfinished ranges so a large PDF does not monopolize the pool.
pub fn jobs(completed: &[Vec<u32>], start: u32, end: u32) -> Result<VecDeque<Job>, String> {
    let mut work = VecDeque::new();
    for (index, range) in ranges(start, end)?.into_iter().enumerate() {
        for (file, saved) in completed.iter().enumerate() {
            if !saved.contains(&(index as u32)) {
                work.push_back(Job {
                    file,
                    index: index as u32,
                    range,
                });
            }
        }
    }
    Ok(work)
}

/// Outcome of one range; interruption never marks the range complete.
#[derive(Debug, PartialEq, Eq)]
pub enum SearchResult {
    /// The authenticated six-digit user password.
    Found(String),
    /// Every candidate in the interval was rejected.
    Exhausted,
    /// Cancellation or another worker stopped this file's search.
    Interrupted,
    /// Authentication failed for a reason other than an incorrect password.
    Error(String),
}

/// Test candidates until success, failure or cancellation, reporting attempt deltas.
/// The shared finished flag stops sibling jobs after success or a PDF error.
pub fn search_range(
    range: Range,
    cancel: &AtomicBool,
    finished: &AtomicBool,
    mut test: impl FnMut(&str) -> Result<bool, String>,
    mut progress: impl FnMut(u64),
) -> SearchResult {
    let mut pending = 0;
    for number in range.start..=range.end {
        if cancel.load(Ordering::Relaxed) || finished.load(Ordering::Relaxed) {
            progress(pending);
            return SearchResult::Interrupted;
        }
        let password = format!("{number:06}");
        pending += 1;
        match test(&password) {
            Ok(true) => {
                finished.store(true, Ordering::Relaxed);
                progress(pending);
                return SearchResult::Found(password);
            }
            Ok(false) => {}
            Err(error) => {
                finished.store(true, Ordering::Relaxed);
                progress(pending);
                return SearchResult::Error(error);
            }
        }
        if pending == 64 {
            progress(pending);
            pending = 0;
        }
    }
    progress(pending);
    SearchResult::Exhausted
}

#[cfg(test)]
/// Pure scheduling and search checks without files, databases or services.
mod tests {
    use super::*;

    #[test]
    /// The final short range includes 2,499 without overlapping earlier ranges.
    fn ranges_cover_inclusive_bounds_without_gaps() {
        assert_eq!(
            ranges(0, 2499).unwrap(),
            vec![
                Range { start: 0, end: 999 },
                Range {
                    start: 1000,
                    end: 1999
                },
                Range {
                    start: 2000,
                    end: 2499
                },
            ]
        );
    }

    #[test]
    /// Reversed ranges and seven-digit candidates must be rejected.
    fn rejects_invalid_numeric_scope() {
        assert!(ranges(10, 9).is_err());
        assert!(ranges(0, 1_000_000).is_err());
    }

    #[test]
    /// The final six-digit candidate is a valid one-element search.
    fn includes_single_last_candidate() {
        assert_eq!(
            ranges(999999, 999999).unwrap(),
            vec![Range {
                start: 999999,
                end: 999999
            }]
        );
    }

    #[test]
    /// Saved checkpoints are excluded while remaining jobs interleave files.
    fn schedules_files_fairly_and_skips_saved_ranges() {
        let work: Vec<_> = jobs(&[vec![0], vec![]], 0, 1999)
            .unwrap()
            .into_iter()
            .map(|j| (j.file, j.index))
            .collect();
        assert_eq!(work, vec![(1, 0), (0, 1), (1, 1)]);
    }

    #[test]
    /// Recovery of 000008 stops the interval after two trials.
    fn finds_zero_padded_password_and_stops_early() {
        let done = AtomicBool::new(false);
        let mut tried = vec![];
        let mut count = 0;
        let result = search_range(
            Range { start: 7, end: 10 },
            &AtomicBool::new(false),
            &done,
            |password| {
                tried.push(password.to_owned());
                Ok(password == "000008")
            },
            |n| count += n,
        );
        assert_eq!(result, SearchResult::Found("000008".into()));
        assert_eq!(tried, vec!["000007", "000008"]);
        assert_eq!(count, 2);
        assert!(done.load(Ordering::Relaxed));
    }

    #[test]
    /// A cancelled interval reports its trial count without claiming exhaustion.
    fn cancel_does_not_complete_a_partial_range() {
        let cancel = AtomicBool::new(false);
        let mut count = 0;
        let result = search_range(
            Range { start: 0, end: 9 },
            &cancel,
            &AtomicBool::new(false),
            |_| {
                cancel.store(true, Ordering::Relaxed);
                Ok(false)
            },
            |n| count += n,
        );
        assert_eq!(result, SearchResult::Interrupted);
        assert_eq!(count, 1);
    }

    #[test]
    /// Unsupported encryption remains an error rather than a password miss.
    fn stops_on_pdf_error_instead_of_reporting_exhaustion() {
        assert_eq!(
            search_range(
                Range { start: 0, end: 9 },
                &AtomicBool::new(false),
                &AtomicBool::new(false),
                |_| Err("Unsupported encryption".into()),
                |_| {}
            ),
            SearchResult::Error("Unsupported encryption".into())
        );
    }

    #[test]
    /// An unsuccessful search still tests 999999.
    fn exhaustion_includes_the_upper_bound() {
        let mut tried = vec![];
        let result = search_range(
            Range {
                start: 999998,
                end: 999999,
            },
            &AtomicBool::new(false),
            &AtomicBool::new(false),
            |password| {
                tried.push(password.to_owned());
                Ok(false)
            },
            |_| {},
        );
        assert_eq!(result, SearchResult::Exhausted);
        assert_eq!(tried, vec!["999998", "999999"]);
    }

    #[test]
    /// Four workers cover 2,500 candidates on each file exactly once.
    fn concurrent_claims_test_each_candidate_once_per_file() {
        use std::sync::atomic::AtomicUsize;
        use std::sync::{Arc, Mutex};
        let queue = Arc::new(Mutex::new(jobs(&[vec![], vec![]], 0, 2499).unwrap()));
        let visits: Arc<Vec<_>> = Arc::new((0..5000).map(|_| AtomicUsize::new(0)).collect());
        let workers: Vec<_> = (0..4)
            .map(|_| {
                let queue = queue.clone();
                let visits = visits.clone();
                std::thread::spawn(move || loop {
                    let Some(job) = queue.lock().unwrap().pop_front() else {
                        break;
                    };
                    let result = search_range(
                        job.range,
                        &AtomicBool::new(false),
                        &AtomicBool::new(false),
                        |password| {
                            let number: usize = password.parse().unwrap();
                            visits[job.file * 2500 + number].fetch_add(1, Ordering::Relaxed);
                            Ok(false)
                        },
                        |_| {},
                    );
                    assert_eq!(result, SearchResult::Exhausted);
                })
            })
            .collect();
        for worker in workers {
            worker.join().unwrap();
        }
        assert!(visits
            .iter()
            .all(|count| count.load(Ordering::Relaxed) == 1));
    }

    #[test]
    /// A sibling's completion prevents any further authentication calls.
    fn a_finished_file_skips_the_entire_remaining_range() {
        let mut count = 0;
        let result = search_range(
            Range { start: 0, end: 9 },
            &AtomicBool::new(false),
            &AtomicBool::new(true),
            |_| panic!("A completed file must not authenticate another password"),
            |n| count += n,
        );
        assert_eq!(result, SearchResult::Interrupted);
        assert_eq!(count, 0);
    }
}
