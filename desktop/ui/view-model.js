/** Derive display values from run snapshots without reading files or history. */

/** Whether the run currently locks selection and settings. */
export function active(run) {
  return run && ['scanning', 'running', 'cancelling'].includes(run.status);
}

/** Count outcomes and derive progress from completed ranges, not repeated trials. */
export function summary(run) {
  const files = run?.files ?? [];
  const rangeSize = run ? run.settings.end - run.settings.start + 1 : 0;
  const complete = files.filter(f => ['found', 'unprotected', 'not_found', 'error'].includes(f.status)).length;
  const covered = files.reduce((total, file) => {
    if (['found', 'unprotected', 'not_found', 'error'].includes(file.status)) return total + rangeSize;
    return total + file.completed.reduce((n, i) => n + Math.min(1000, rangeSize - i * 1000), 0);
  }, 0);
  return {
    total: files.length, complete,
    found: files.filter(f => f.status === 'found').length,
    errors: files.filter(f => f.status === 'error').length,
    attempts: files.reduce((n, f) => n + f.attempts, 0),
    percent: files.length ? Math.min(100, Math.floor(covered / (rangeSize * files.length) * 100)) : 0,
  };
}

/** Match source paths and status labels without case sensitivity. */
export function matches(run, query) {
  return [run.status, ...run.inputs, ...run.files.map(f => `${f.path} ${f.status}`)]
    .join(' ').toLowerCase().includes(query.trim().toLowerCase());
}

/** Extract a display name from either Windows or Unix path separators. */
export function basename(path) {
  return path.replace(/\\/g, '/').split('/').filter(Boolean).pop() ?? path;
}

/** Format elapsed milliseconds as whole seconds or minutes and seconds. */
export function duration(ms) {
  const seconds = Math.floor(ms / 1000);
  return seconds < 60 ? `${seconds}s` : `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
}
