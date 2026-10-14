/** Pure display checks with worked range counts and local path examples. */
import test from 'node:test';
import assert from 'node:assert/strict';
import { summary, matches, basename, active } from './view-model.js';

/** A cancelled 2,500-candidate run with one complete range and 80 partial trials. */
const run = { inputs: ['C:\\Bank\\June.pdf'], status: 'cancelled', settings: { start: 0, end: 2499 },
  files: [{ path: 'C:\\Bank\\June.pdf', status: 'cancelled', attempts: 1080, completed: [0] }] };

test('progress reflects saved coverage rather than partial repeated attempts', () => {
  assert.deepEqual(summary(run), { total: 1, complete: 0, found: 0, errors: 0, attempts: 1080, percent: 40 });
});
test('last shorter range contributes only its own size', () => {
  assert.equal(summary({ ...run, files: [{ ...run.files[0], completed: [2] }] }).percent, 20);
});
test('terminal outcomes count as processed without claiming all passwords were tried', () => {
  const result = summary({ ...run, files: [{ ...run.files[0], status: 'found', password: '001234', attempts: 1 }] });
  assert.equal(result.percent, 100); assert.equal(result.found, 1); assert.equal(result.attempts, 1);
});
test('history search matches input paths and status without case sensitivity', () => {
  assert.ok(matches(run, ' JUNE ')); assert.ok(matches(run, 'cancelled')); assert.ok(!matches(run, 'July'));
});
test('file labels support Windows and Unix paths', () => {
  assert.equal(basename('C:\\Bank\\June.pdf'), 'June.pdf'); assert.equal(basename('/bank/June.pdf'), 'June.pdf');
});
test('only live states lock the recovery form', () => {
  assert.ok(active({ status: 'cancelling' })); assert.ok(!active({ status: 'cancelled' }));
});
