/** Connect the recovery and history views to local Tauri commands. */
import { active, summary, matches, basename, duration } from './view-model.js';

/** Find a required control by its stable HTML ID. */
const $ = id => document.getElementById(id);
/** Labels for persisted run states and per-file outcomes. */
const labels = { scanning: 'Scanning PDFs', running: 'Recovering', cancelling: 'Cancelling', cancelled: 'Cancelled',
  interrupted: 'Interrupted', completed: 'Completed', completed_with_errors: 'Completed with errors', error: 'Error',
  queued: 'Queued', found: 'Recovered', unprotected: 'No password needed', not_found: 'Not found in range' };
/** Format counts using the desktop's locale. */
const number = value => value.toLocaleString();
/** Selected inputs, displayed runs and request flags shared by view handlers. */
let inputs = [], current = null, history = [], selectedRun = null, starting = false, pollBusy = false, pollPending = false, historyLoaded = false;

/** Invoke a local command or explain why a standalone browser cannot run it. */
function invoke(command, args) {
  if (!window.__TAURI__?.core) return Promise.reject(new Error('Open Brutus as a desktop app to select files and run recovery.'));
  return window.__TAURI__.core.invoke(command, args);
}

/** Build UI nodes using textContent so source paths cannot become HTML. */
function element(tag, className, text) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}

/** Announce state changes without moving keyboard focus. */
function announce(text) { $('announcement').textContent = text; }
/** Show errors inside an open detail dialog or in the main workspace. */
function message(error) {
  if ($('run-dialog').open) {
    let notice = $('detail-content').querySelector('.dialog-error');
    if (!notice) { notice = element('div', 'run-warning dialog-error'); notice.setAttribute('role', 'alert'); $('detail-content').prepend(notice); }
    notice.textContent = String(error?.message ?? error);
    return;
  }
  const box = $('message'); box.replaceChildren(element('span', '', String(error?.message ?? error)));
  const close = element('button', '', '×'); close.type = 'button'; close.setAttribute('aria-label', 'Dismiss message');
  close.addEventListener('click', () => { box.hidden = true; }); box.append(close); box.hidden = false;
}

/** Switch views, move focus to the heading and refresh history when selected. */
function navigate(view) {
  const recovery = view === 'recovery';
  $('recovery-view').hidden = !recovery; $('history-view').hidden = recovery;
  for (const name of ['recovery', 'history']) {
    const selected = view === name;
    $(`nav-${name}`).classList.toggle('selected', selected);
    if (selected) $(`nav-${name}`).setAttribute('aria-current', 'page'); else $(`nav-${name}`).removeAttribute('aria-current');
  }
  $('breadcrumb').textContent = `Workspace / ${recovery ? 'Recovery' : 'History'}`;
  $(`${view}-title`).focus();
  if (!recovery) refreshHistory();
}

/** Show selected paths and allow removal only when recovery is idle. */
function renderInputs() {
  const list = $('inputs'); list.replaceChildren();
  inputs.forEach((path, index) => {
    const row = element('div', 'input-row');
    const icon = element('span', 'accent', '↗'); icon.setAttribute('aria-hidden', 'true');
    const text = element('div', 'input-path'); text.append(element('strong', '', basename(path)), element('small', '', path)); text.title = path;
    const remove = element('button', 'icon-button', '×'); remove.type = 'button'; remove.setAttribute('aria-label', `Remove ${basename(path)}`);
    remove.disabled = !!active(current) || starting;
    remove.addEventListener('click', () => { inputs.splice(index, 1); renderInputs(); $('pick-files').focus(); });
    row.append(icon, text, remove); list.append(row);
  });
  $('input-count').textContent = inputs.length ? `${inputs.length} input${inputs.length === 1 ? '' : 's'} selected` : 'No inputs selected';
  $('clear-inputs').hidden = !inputs.length;
  lockControls();
}

/** Keep selection and settings fixed while starting or running recovery. */
function lockControls() {
  const locked = !!active(current) || starting;
  for (const id of ['pick-files', 'pick-folder', 'clear-inputs', 'workers', 'range-start', 'range-end']) $(id).disabled = locked;
  $('start-run').disabled = locked || !inputs.length;
  $('start-run').firstChild.textContent = starting ? 'Starting… ' : 'Start recovery ';
  $('cancel-run').hidden = !active(current); $('cancel-run').disabled = current?.status === 'cancelling';
  $('resume-current').hidden = !current || !['cancelled', 'interrupted'].includes(current.status);
  $('resume-current').disabled = locked;
  document.querySelectorAll('.input-row button').forEach(button => { button.disabled = locked; });
}

/** Add native picker results without duplicating identical selected paths. */
async function pick(folder) {
  try {
    const paths = await invoke('pick_input', { folder });
    inputs = [...new Set([...inputs, ...paths])]; renderInputs();
    if (paths.length) announce(`${paths.length} input${paths.length === 1 ? '' : 's'} selected.`);
  } catch (error) { message(error); }
}

/** Refresh the worker label and inclusive range budget before validation. */
function updateSettings() {
  $('worker-value').value = $('workers').value;
  const start = Number($('range-start').value), end = Number($('range-end').value);
  $('candidate-count').textContent = start <= end ? number(end - start + 1) : 'Invalid range';
  $('range-end').setCustomValidity(start > end ? 'The end must be greater than or equal to the start.' : '');
}

/** Start validated recovery and retain inputs if the command fails. */
async function start(event) {
  event.preventDefault(); updateSettings();
  if (!$('recovery-form').reportValidity()) return;
  starting = true; lockControls(); $('message').hidden = true;
  try {
    await invoke('start_run', { inputs, settings: { start: Number($('range-start').value), end: Number($('range-end').value), workers: Number($('workers').value) } });
    await poll(); announce('Recovery started.');
  } catch (error) { message(error); }
  finally { starting = false; lockControls(); }
}

/** Render the visible label for a saved run or file status. */
function badge(status) { return element('span', `badge ${status}`, labels[status] ?? status); }

/** Display outcomes and expose copy controls only for recovered passwords. */
function fileRows(files) {
  const container = element('div', 'file-results');
  for (const file of files) {
    const row = element('div', 'file-row');
    const left = element('div', 'input-path');
    const name = element('span', 'file-name', basename(file.path)); name.title = file.path;
    const detail = file.error ?? (file.reused ? 'Reused a saved result for identical content' : `${number(file.attempts)} passwords tried · ${file.path}`);
    left.append(name, element('span', 'file-info', detail));
    const right = element('div', 'result-right'); right.append(badge(file.status));
    if (file.status === 'found' && file.password !== null && file.password !== undefined) {
      right.append(element('code', 'password', file.password));
      const copy = element('button', 'copy-button', 'Copy'); copy.type = 'button'; copy.setAttribute('aria-label', `Copy password for ${basename(file.path)}`);
      copy.addEventListener('click', async () => {
        try { await navigator.clipboard.writeText(file.password); copy.textContent = 'Copied'; announce('Password copied.'); }
        catch { message('Could not copy the password. Select the displayed password and copy it manually.'); }
      }); right.append(copy);
    }
    row.append(left, right); container.append(row);
  }
  return container;
}

/** Update metrics while retaining password selection and copy-button focus. */
function renderCurrent(previous) {
  lockControls();
  if (!current) return;
  $('run-empty').hidden = true; $('run-content').hidden = false;
  const stats = summary(current);
  $('metric-files').textContent = `${stats.complete} / ${stats.total}`;
  $('metric-found').textContent = stats.found; $('metric-attempts').textContent = number(stats.attempts);
  $('metric-time').textContent = duration(current.elapsedMs);
  $('progress-label').textContent = current.status === 'scanning' ? 'Scanning and preparing PDFs…' : `${labels[current.status]} · ${current.settings.workers} workers`;
  $('progress-percent').textContent = `${stats.percent}%`; $('run-progress').value = stats.percent;
  const indicator = $('run-indicator'); indicator.replaceChildren(element('span'), document.createTextNode(labels[current.status]));
  indicator.classList.toggle('busy', !!active(current)); indicator.classList.toggle('error', current.status === 'error' || current.status === 'completed_with_errors');
  const errors = $('run-errors'); errors.replaceChildren();
  if (current.error) errors.append(element('div', 'run-warning', current.error));
  for (const warning of current.warnings) errors.append(element('div', 'run-warning', warning));
  const results = $('file-results');
  // Keep password selection and keyboard focus stable during progress updates.
  const signature = JSON.stringify(current.files.map(f => [f.path, f.status, f.password, f.error, f.reused]));
  if (results.dataset.signature !== signature) {
    const focusLabel = results.contains(document.activeElement) ? document.activeElement.getAttribute('aria-label') : null;
    results.replaceChildren(...fileRows(current.files).children); results.dataset.signature = signature;
    if (focusLabel) [...results.querySelectorAll('button')].find(button => button.getAttribute('aria-label') === focusLabel)?.focus();
  } else {
    current.files.forEach((file, i) => {
      if (!file.error && !file.reused) results.children[i].querySelector('.file-info').textContent = `${number(file.attempts)} passwords tried · ${file.path}`;
    });
  }
  if (current.status !== previous?.status) announce(labels[current.status]);
}

/** Serialize snapshot requests and queue a retry when callers overlap. */
async function poll() {
  if (pollBusy) { pollPending = true; return; }
  pollBusy = true;
  try {
    const previous = current; current = await invoke('get_current'); renderCurrent(previous);
    if (current && !active(current) && (active(previous) || current.id !== previous?.id)) await refreshHistory();
  } catch (error) { message(error); }
  finally {
    pollBusy = false;
    if (pollPending) { pollPending = false; queueMicrotask(poll); }
  }
}

/** Load saved runs and keep failures visible to the user. */
async function refreshHistory() {
  try {
    history = await invoke('get_history'); historyLoaded = true;
    $('history-count').textContent = history.length; renderHistory();
  } catch (error) { $('history-summary').textContent = 'History could not be loaded'; message(error); }
}

/** Filter saved runs by source paths and outcome, including an empty state. */
function renderHistory() {
  const list = $('history-list'); list.replaceChildren();
  const shown = history.filter(run => matches(run, $('history-search').value));
  $('history-summary').textContent = `${shown.length} of ${history.length} runs`;
  if (!shown.length) {
    const empty = element('div', 'card empty-state'); const text = element('div');
    text.append(element('h3', '', history.length ? 'No matching runs.' : 'No runs yet.'),
      element('p', '', history.length ? 'Try another file name or outcome.' : 'Your first recovery run will be saved here.'));
    empty.append(text); list.append(empty); return;
  }
  for (const run of shown) {
    const button = element('button', 'history-row'); button.type = 'button';
    const info = element('div', 'history-run');
    const title = run.inputs.map(basename).join(', ');
    info.append(element('strong', '', title), element('small', '', `${new Date(run.startedAt).toLocaleString()} · ${run.files.length} PDFs · ${duration(run.elapsedMs)}`));
    info.title = run.inputs.join('\n');
    button.append(info, element('span', 'history-facts', `${summary(run).found} recovered`), badge(run.status), element('span', 'history-arrow', '↗'));
    button.addEventListener('click', () => showDetails(run)); list.append(button);
  }
}

/** Open saved settings, timestamps and file outcomes in a modal dialog. */
function showDetails(run) {
  selectedRun = run; $('detail-title').textContent = run.inputs.map(basename).join(', ');
  const content = $('detail-content'); content.replaceChildren();
  const meta = element('div', 'detail-meta');
  meta.append(badge(run.status), element('div', '', `Started ${new Date(run.startedAt).toLocaleString()}`),
    element('div', '', run.finishedAt ? `Finished ${new Date(run.finishedAt).toLocaleString()}` : 'Not finished'),
    element('div', '', `${duration(run.elapsedMs)} active time · ${run.settings.workers} workers · ${String(run.settings.start).padStart(6, '0')} to ${String(run.settings.end).padStart(6, '0')}`));
  const paths = element('ul'); run.inputs.forEach(path => paths.append(element('li', '', path))); meta.append(paths); content.append(meta);
  if (run.error) content.append(element('div', 'run-warning', run.error));
  run.warnings.forEach(warning => content.append(element('div', 'run-warning', warning)));
  content.append(fileRows(run.files));
  $('resume-history').hidden = !['cancelled', 'interrupted'].includes(run.status);
  $('resume-history').disabled = !!active(current) || starting;
  $('run-dialog').showModal();
}

/** Resume a saved run and return to recovery after the command succeeds. */
async function resume(id) {
  starting = true; lockControls();
  try {
    await invoke('resume_run', { id }); $('run-dialog').close(); navigate('recovery'); await poll(); announce('Recovery resumed.');
  } catch (error) { message(error); }
  finally { starting = false; lockControls(); }
}

$('pick-files').addEventListener('click', () => pick(false)); $('pick-folder').addEventListener('click', () => pick(true));
$('clear-inputs').addEventListener('click', () => { inputs = []; renderInputs(); $('pick-files').focus(); });
$('recovery-form').addEventListener('submit', start);
['workers', 'range-start', 'range-end'].forEach(id => $(id).addEventListener('input', updateSettings));
['range-start', 'range-end'].forEach(id => $(id).addEventListener('blur', () => { if ($(id).value) $(id).value = $(id).value.padStart(6, '0'); }));
$('nav-recovery').addEventListener('click', () => navigate('recovery')); $('nav-history').addEventListener('click', () => navigate('history'));
$('refresh-history').addEventListener('click', refreshHistory); $('history-search').addEventListener('input', renderHistory);
$('cancel-run').addEventListener('click', async () => { try { await invoke('cancel_run'); await poll(); } catch (error) { message(error); } });
$('resume-current').addEventListener('click', () => resume(current.id)); $('resume-history').addEventListener('click', () => resume(selectedRun.id));
['close-dialog', 'dismiss-dialog'].forEach(id => $(id).addEventListener('click', () => $('run-dialog').close()));
$('run-dialog').addEventListener('close', () => { selectedRun = null; });

/** Load defaults and history, then poll while recovery is active or starting. */
async function initialize() {
  try {
    const defaults = await invoke('get_defaults'); $('workers').value = defaults.workers;
    $('worker-caption').textContent = `${defaults.workers} suggested`; updateSettings();
    await refreshHistory(); await poll();
  } catch (error) { message(error); $('worker-caption').textContent = '1–32 workers'; }
  if (!historyLoaded) $('history-summary').textContent = 'History unavailable';
  renderInputs();
  if (window.__TAURI__?.core) setInterval(() => { if (active(current) || starting) poll(); }, 500);
}
initialize();
