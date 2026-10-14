import { readFile } from 'node:fs/promises';
import assert from 'node:assert/strict';
import test from 'node:test';
import { formatTime, formatSent, parseSent, linkSegments } from '../../ui/format.js';

const parity = JSON.parse(await readFile(new URL('../parity.json', import.meta.url), 'utf8'));
const suites = {
  format_time: (item) => formatTime(item.iso),
  format_sent: (item) => formatSent(item.sent),
  parse_sent: (item) => parseSent(item.raw, item.order),
  linkify: (item) => linkSegments(item.text),
};
for (const [name, run] of Object.entries(suites)) {
  for (const [index, item] of parity[name].entries()) {
    test(`${name} ${index + 1}: ${JSON.stringify(item)}`, () => {
      assert.deepEqual(run(item), item.expected);
    });
  }
}
