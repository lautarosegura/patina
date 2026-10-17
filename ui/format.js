const pad = (value) => String(value).padStart(2, '0');

function validDate(year, month, day, hour, minute, second = 0) {
  const leap = year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0);
  const days = [31, leap ? 29 : 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
  return year >= 1 && year <= 9999 && month >= 1 && month <= 12 &&
    day >= 1 && day <= days[month - 1] && hour >= 0 && hour <= 23 &&
    minute >= 0 && minute <= 59 && second >= 0 && second <= 59;
}

export function formatTime(iso) {
  const match = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2})(?::(\d{2})(?:\.\d+)?)?$/.exec(iso);
  if (!match || !validDate(...match.slice(1).map((part) => Number(part || 0)))) {
    throw new RangeError(`Fecha ISO inválida: ${iso}`);
  }
  return `${match[3]}/${match[2]} ${match[4]}:${match[5]}`;
}

export function parseSent(raw, order = '') {
  const time = /(\d{1,2})[:.](\d{2})\s*([ap])?\.?\s*m?\.?/i.exec(raw);
  const date = /(\d{1,4})[/.-](\d{1,2})[/.-](\d{1,4})/.exec(raw);
  if (!time || !date) return '';
  let hour = Number(time[1]);
  const minute = Number(time[2]);
  if (time[3]) hour = hour % 12 + (time[3].toLowerCase() === 'p' ? 12 : 0);
  const [a, b, c] = date.slice(1).map(Number);
  const guessedOrder = order || (a > 31 ? 'ymd' : b > 12 ? 'mdy' : 'dmy');
  const parts = { dmy: [c, b, a], mdy: [c, a, b], ymd: [a, b, c] }[guessedOrder];
  if (!parts) return '';
  let [year, month, day] = parts;
  if (year <= 99) year += 2000;
  if (!validDate(year, month, day, hour, minute)) return '';
  return `${String(year).padStart(4, '0')}-${pad(month)}-${pad(day)}T${pad(hour)}:${pad(minute)}`;
}

export function formatSent(sent) {
  try {
    return formatTime(sent);
  } catch (error) {
    if (!(error instanceof RangeError)) throw error;
    const iso = parseSent(sent);
    return iso ? formatTime(iso) : sent;
  }
}

export function linkSegments(text) {
  const segments = [];
  let position = 0;
  for (const match of text.matchAll(/https?:\/\/[^\s<>"']+/g)) {
    const url = match[0].replace(/[.,;:!?\)\]}]+$/, '');
    if (match.index > position) segments.push({ text: text.slice(position, match.index) });
    segments.push({ link: url });
    position = match.index + url.length;
  }
  if (position < text.length) segments.push({ text: text.slice(position) });
  return segments;
}

const MARKS = ['bold', 'italic', 'code'];
const markKey = (style) => style ? `${MARKS.filter((mark) => style[mark]).join()}|${style.href || ''}` : '|';

// Lays the formatting of `runs` ({text, bold, italic, code, href}, parsed from the captured HTML)
// over `text`, which stays exactly as saved. Returns null when there's nothing to show or the
// characters differ apart from whitespace, so the item renders as plain text.
export function richSegments(text, runs) {
  const marked = [];
  for (const run of runs) {
    for (const character of run.text) if (!/\s/.test(character)) marked.push({ character, style: run });
  }
  const styles = [];
  let next = 0;
  for (const character of text) {
    if (/\s/.test(character)) {
      styles.push(undefined);
      continue;
    }
    if (marked[next]?.character !== character) return null;
    styles.push(marked[next++].style);
  }
  if (next !== marked.length || !next) return null;
  // A mark on every character tells nothing about the text (a heading, a terminal page): drop it.
  const shown = Object.fromEntries(MARKS.map((mark) => [mark, !marked.every(({ style }) => style[mark])]));
  const characters = Array.from(text);
  const visible = styles.map((style) => style && {
    ...Object.fromEntries(MARKS.map((mark) => [mark, shown[mark] && !!style[mark]])),
    href: style.href || '',
  });
  // Whitespace takes the style around it only when both sides agree, so "**a** b" keeps the space plain.
  for (let i = 0; i < visible.length; i++) {
    if (visible[i]) continue;
    let after = i;
    while (after < visible.length && !visible[after]) after++;
    const before = visible[i - 1];
    const style = before && visible[after] && markKey(before) === markKey(visible[after]) ? before : null;
    for (; i < after; i++) visible[i] = style;
    i--;
  }
  const segments = [];
  characters.forEach((character, i) => {
    const style = visible[i] || null;
    const last = segments.at(-1);
    if (last && markKey(last.style) === markKey(style)) last.text += character;
    else segments.push({ text: character, style });
  });
  if (segments.every(({ style }) => markKey(style) === '|')) return null;
  return segments.map(({ text, style }) => ({
    text,
    ...Object.fromEntries(MARKS.map((mark) => [mark, !!style?.[mark]])),
    href: style?.href || '',
  }));
}
