import { readFile } from 'node:fs/promises';
import assert from 'node:assert/strict';
import test from 'node:test';
import { formatTime, formatSent, parseSent, linkSegments, richSegments } from '../../ui/format.js';

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

const plain = (text) => ({ text, bold: false, italic: false, code: false, href: '' });

test('richSegments lays bold and links over the saved text', () => {
  const runs = [{ text: 'Use ' }, { text: 'TOML as the default', bold: true }, { text: ', see ' }, { text: 'docs', href: 'https://toml.io/' }];
  assert.deepEqual(richSegments('Use TOML as  the default, see docs', runs), [
    plain('Use '),
    { ...plain('TOML as  the default'), bold: true },
    plain(', see '),
    { ...plain('docs'), href: 'https://toml.io/' },
  ]);
});

test('richSegments keeps the saved line breaks and leaves the space between marks plain', () => {
  const runs = [{ text: 'uno', bold: true }, { text: ' ' }, { text: 'dos', italic: true }];
  assert.deepEqual(richSegments('uno\n\ndos', runs), [{ ...plain('uno'), bold: true }, plain('\n\n'), { ...plain('dos'), italic: true }]);
});

test('richSegments gives up when the characters differ or nothing is formatted', () => {
  assert.equal(richSegments('hola mundo', [{ text: 'hola', bold: true }]), null);
  assert.equal(richSegments('hola', [{ text: 'chau', bold: true }]), null);
  assert.equal(richSegments('hola mundo', [{ text: 'hola mundo' }]), null);
  assert.equal(richSegments('', []), null);
});

test('richSegments drops a mark that covers every character', () => {
  assert.equal(richSegments('Título entero', [{ text: 'Título entero', bold: true }]), null);
  const runs = [{ text: 'ls ', code: true }, { text: '-la', code: true, bold: true }];
  assert.deepEqual(richSegments('ls -la', runs), [plain('ls '), { ...plain('-la'), bold: true }]);
});
