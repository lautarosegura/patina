import { formatTime, formatSent, linkSegments, richSegments } from './format.js';

const tauri = window.__TAURI__;
const $ = (selector) => document.querySelector(selector);
const entry = $('#prompt-entry');
const search = $('#search');
const menu = $('#menu');
const menuButton = $('#menu-button');
const notes = $('#notes-text');
const lists = { queue: $('#queue-list'), history: $('#history-list') };
const selection = { queue: -1, history: -1 };
const queries = { queue: '', history: '' };
const THEME_KEY = 'patina-theme';
const darkScheme = window.matchMedia('(prefers-color-scheme: dark)');
const copiedUntil = new Map();
const copyTimers = new Map();
const favicons = new Map();
let state = { items: [], history: [], notes: '' };
let activeTab = 'queue';
let operations = Promise.resolve();
let notesTimer;
let notesRevision = 0;
let savedNotesRevision = 0;

const icons = {
  check: '<path d="m3 8 3 3 7-7" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>',
  undo: '<path d="m6 3-4 4 4 4V8h3a3 3 0 1 1 0 6" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>',
  globe: '<circle cx="8" cy="8" r="5.5" fill="none" stroke="currentColor"/><ellipse cx="8" cy="8" rx="2.5" ry="5.5" fill="none" stroke="currentColor"/><path d="M2.5 8h11" fill="none" stroke="currentColor"/>',
};

function icon(name, className = '') {
  const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  svg.setAttribute('viewBox', '0 0 16 16');
  svg.setAttribute('aria-hidden', 'true');
  if (className) svg.setAttribute('class', className);
  svg.innerHTML = icons[name];
  return svg;
}

function element(tag, className, text) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}

function run(operation) {
  const next = operations.then(operation);
  operations = next.catch((error) => console.error('Patina:', error));
  return next;
}

function invoke(name, args = {}) {
  return tauri.core.invoke(name, args);
}

function mutate(name, args = {}, after) {
  return run(async () => {
    const result = await invoke(name, args);
    await reload();
    after?.(result);
    return result;
  });
}

function pythonCasefold(text) {
  return Array.from(text.toLowerCase(), (character) =>
    character === 'ı' ? character : character.toUpperCase().toLowerCase()).join('');
}

function visibleItems(tab) {
  const query = pythonCasefold(queries[tab].trim());
  const items = tab === 'queue' ? state.items : [...state.history].reverse();
  return items.filter((item) => !query ||
    pythonCasefold(['text', 'source', 'sender', 'chat', 'host', 'title']
      .map((key) => item[key] || '').join(' ')).includes(query));
}

function rows(tab) {
  return [...lists[tab].querySelectorAll('.row')];
}

function applyTheme() {
  const choice = localStorage.getItem(THEME_KEY) || 'system';
  const theme = choice === 'system' ? (darkScheme.matches ? 'dark' : 'light') : choice;
  document.documentElement.dataset.theme = theme;
  for (const item of menu.querySelectorAll('[data-theme]')) item.setAttribute('aria-checked', String(item.dataset.theme === choice));
}

function openMenu(open) {
  menu.hidden = !open;
  menuButton.setAttribute('aria-expanded', String(open));
  if (open) (menu.querySelector('[aria-checked="true"]') || menu.querySelector('.menu-item')).focus();
}

function select(tab, index, focus = true) {
  const tabRows = rows(tab);
  selection[tab] = tabRows.length ? Math.max(0, Math.min(index, tabRows.length - 1)) : -1;
  tabRows.forEach((row, i) => {
    const selected = i === selection[tab];
    row.classList.toggle('selected', selected);
    row.setAttribute('aria-selected', String(selected));
    row.tabIndex = selected ? 0 : -1;
  });
  if (focus && selection[tab] >= 0) {
    tabRows[selection[tab]].focus({ preventScroll: true });
    tabRows[selection[tab]].scrollIntoView({ block: 'nearest' });
  }
}

function setTab(tab, focus = false) {
  activeTab = tab;
  for (const view of ['queue', 'notes', 'history']) $(`#${view}`).hidden = view !== tab;
  for (const item of menu.querySelectorAll('[data-view]')) item.setAttribute('aria-checked', String(item.dataset.view === tab));
  $('#back').hidden = tab === 'queue';
  $('#search-wrap').hidden = tab === 'notes';
  $('#view-title').hidden = tab !== 'notes';
  if (tab !== 'notes') {
    search.value = queries[tab];
    search.placeholder = tab === 'queue' ? 'Buscar' : 'Buscar en el historial';
  }
  if (focus) ({ queue: entry, notes, history: search })[tab].focus();
}

function stopRowEvents(node) {
  node.addEventListener('mousedown', (event) => {
    event.stopPropagation();
    event.preventDefault();
  });
  node.addEventListener('dblclick', (event) => event.stopPropagation());
}

function link(text, url, className) {
  const node = element('a', className, text);
  node.href = url;
  node.tabIndex = -1;
  stopRowEvents(node);
  node.addEventListener('click', (event) => {
    event.preventDefault();
    event.stopPropagation();
    void run(() => invoke('open_url', { url }));
  });
  return node;
}

function favicon(item) {
  const fallback = icon('globe', 'meta-icon');
  if (!item.favicon) return fallback;
  if (!favicons.has(item.favicon)) {
    favicons.set(item.favicon, invoke('favicon_url', { name: item.favicon }).catch((error) => {
      console.error('Patina favicon:', error);
      return null;
    }));
  }
  favicons.get(item.favicon).then((url) => {
    if (!url || !url.startsWith('data:')) return;
    const image = element('img', 'meta-icon');
    image.alt = '';
    image.src = url;
    image.addEventListener('error', () => image.replaceWith(fallback), { once: true });
    fallback.replaceWith(image);
  });
  return fallback;
}

function appendLinked(parent, text) {
  for (const segment of linkSegments(text)) {
    parent.append(segment.link === undefined ? document.createTextNode(segment.text) : link(segment.link, segment.link));
  }
}

const BOLD_TAGS = new Set(['B', 'STRONG', 'H1', 'H2', 'H3', 'H4', 'H5', 'H6', 'TH']);
const ITALIC_TAGS = new Set(['I', 'EM', 'CITE']);
const CODE_TAGS = new Set(['CODE', 'KBD', 'SAMP', 'TT', 'PRE']);
const SKIPPED_TAGS = new Set(['HEAD', 'SCRIPT', 'STYLE', 'TEMPLATE', 'NOSCRIPT']);

// The closest element that decides a mark wins, so <b><span style="font-weight:400"> is plain.
function htmlRuns(html) {
  const body = new DOMParser().parseFromString(html, 'text/html').body;
  const walker = document.createTreeWalker(body, NodeFilter.SHOW_TEXT);
  const runs = [];
  for (let node = walker.nextNode(); node; node = walker.nextNode()) {
    const run = { text: node.data };
    let skipped = false;
    for (let el = node.parentElement; el && el !== body; el = el.parentElement) {
      if (SKIPPED_TAGS.has(el.tagName)) skipped = true;
      const weight = el.style.fontWeight;
      if (run.bold === undefined && (BOLD_TAGS.has(el.tagName) || weight)) {
        run.bold = weight ? weight === 'bold' || weight === 'bolder' || Number(weight) >= 600 : true;
      }
      if (run.italic === undefined && (ITALIC_TAGS.has(el.tagName) || el.style.fontStyle)) {
        run.italic = el.style.fontStyle ? el.style.fontStyle === 'italic' : true;
      }
      if (run.code === undefined && (CODE_TAGS.has(el.tagName) || /mono|courier|consolas|menlo/i.test(el.style.fontFamily))) run.code = true;
      if (run.href === undefined && el.tagName === 'A' && /^https?:\/\//.test(el.getAttribute('href') || '')) run.href = el.href;
    }
    if (!skipped) runs.push(run);
  }
  return runs;
}

function itemText(item) {
  const text = element('div', 'item-text');
  let segments = null;
  if (item.html) {
    try {
      segments = richSegments(item.text, htmlRuns(item.html));
    } catch (error) {
      console.error('Patina formato:', error);
    }
  }
  if (!segments) {
    appendLinked(text, item.text);
    return text;
  }
  for (const segment of segments) {
    const marks = ['bold', 'italic', 'code'].filter((mark) => segment[mark]);
    const node = segment.href ? link(segment.text, segment.href) : marks.length ? element('span') : text;
    node.classList?.add(...marks);
    if (segment.href) text.append(node);
    else if (node === text) appendLinked(text, segment.text);
    else {
      appendLinked(node, segment.text);
      text.append(node);
    }
  }
  return text;
}

function itemColumn(item) {
  const column = element('div', 'item-column');
  column.append(itemText(item));
  const archived = item.archived ? ` · archivado ${formatTime(item.archived)}` : '';
  if (item.sender) {
    const line = element('div', 'meta-line');
    line.title = item.source || 'WhatsApp';
    const logo = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    logo.setAttribute('viewBox', '0 0 24 24');
    logo.setAttribute('class', 'meta-icon');
    logo.setAttribute('aria-hidden', 'true');
    logo.innerHTML = WHATSAPP_PATH;
    const sender = item.sender + (item.chat ? ` (${item.chat})` : '');
    line.append(logo, element('span', 'item-meta', `${sender} · ${item.sent ? formatSent(item.sent) : formatTime(item.created)}${archived}`));
    column.append(line);
  } else if (item.host) {
    const line = element('div', 'meta-line');
    line.title = [item.title, item.url].filter(Boolean).join('\n');
    const meta = element('div', 'item-meta');
    meta.append(item.url ? link(item.host, item.url, 'host') : document.createTextNode(item.host),
      document.createTextNode(` · ${formatTime(item.created)}${archived}`));
    line.append(favicon(item), meta);
    column.append(line);
  } else {
    column.append(element('div', 'item-meta', [item.kind === 'capture' ? 'captura' : 'prompt', item.source, formatTime(item.created)]
      .filter(Boolean).join(' · ') + archived));
  }
  return column;
}

function showCopied(id) {
  for (const button of document.querySelectorAll('.copy-button')) {
    if (button.dataset.id === id) button.classList.toggle('copied', (copiedUntil.get(id) || 0) > Date.now());
  }
}

function copyButton(item) {
  const button = element('button', 'copy-button');
  button.type = 'button';
  button.tabIndex = -1;
  button.dataset.id = item.id;
  button.classList.toggle('copied', (copiedUntil.get(item.id) || 0) > Date.now());
  const labels = element('span', 'copy-labels');
  const confirmation = element('span', 'copy-label confirmation');
  confirmation.append(icon('check'), document.createTextNode('¡Copiado!'));
  confirmation.setAttribute('aria-hidden', 'true');
  labels.append(element('span', 'copy-label idle', 'Copiar'), confirmation);
  button.append(labels);
  stopRowEvents(button);
  button.addEventListener('click', (event) => {
    event.stopPropagation();
    void run(async () => {
      await invoke('copy_text', { text: item.text, hideAfterCopy: false });
      copiedUntil.set(item.id, Date.now() + 1400);
      clearTimeout(copyTimers.get(item.id));
      showCopied(item.id);
      copyTimers.set(item.id, setTimeout(() => {
        copiedUntil.delete(item.id);
        copyTimers.delete(item.id);
        showCopied(item.id);
      }, 1400));
    });
  });
  return button;
}

function row(item, tab, index) {
  const node = element('div', `row ${tab}-row${item.kind === 'capture' ? ' capture' : ''}${tab === 'queue' && item.done ? ' done' : ''}`);
  node.setAttribute('role', 'option');
  node.setAttribute('aria-selected', 'false');
  node.dataset.id = item.id;
  node.tabIndex = -1;
  if (tab === 'queue') {
    const check = element('input', 'check');
    check.type = 'checkbox';
    check.checked = item.done;
    check.tabIndex = -1;
    check.setAttribute('aria-label', 'Marcar como completado');
    check.addEventListener('mousedown', (event) => { event.preventDefault(); event.stopPropagation(); });
    check.addEventListener('click', (event) => event.stopPropagation());
    check.addEventListener('dblclick', (event) => event.stopPropagation());
    check.addEventListener('change', () => void mutate('set_done', { id: item.id, done: check.checked }));
    node.append(check);
  }
  node.append(itemColumn(item), copyButton(item));
  if (tab === 'history') {
    const restore = element('button', 'icon-button restore');
    restore.type = 'button';
    restore.title = 'Devolver a la cola (Ctrl+Z)';
    restore.setAttribute('aria-label', 'Devolver a la cola');
    restore.append(icon('undo'));
    stopRowEvents(restore);
    restore.addEventListener('click', (event) => {
      event.stopPropagation();
      void mutate('restore_item', { id: item.id });
    });
    node.append(restore);
  }
  node.addEventListener('click', () => select(tab, index));
  node.addEventListener('focus', () => select(tab, index, false));
  node.addEventListener('dblclick', () => void run(() => invoke('copy_text', { text: item.text, hideAfterCopy: true })));
  return node;
}

const DAY = 86400000;

function dayLabel(iso) {
  const day = new Date(`${iso.slice(0, 10)}T00:00`);
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const ago = Math.round((today - day) / DAY);
  if (ago === 0) return 'Hoy';
  if (ago === 1) return 'Ayer';
  return day.toLocaleDateString('es', { day: 'numeric', month: 'short', year: day.getFullYear() === today.getFullYear() ? undefined : 'numeric' });
}

function renderList(tab) {
  const items = visibleItems(tab);
  const fragment = document.createDocumentFragment();
  let day = '';
  items.forEach((item, index) => {
    // The history is grouped by the day each item left the queue.
    const itemDay = tab === 'history' ? (item.archived || item.created).slice(0, 10) : '';
    if (itemDay && itemDay !== day) {
      day = itemDay;
      const section = element('div', 'section', dayLabel(itemDay));
      section.setAttribute('role', 'presentation');
      fragment.append(section);
    }
    fragment.append(row(item, tab, index));
  });
  lists[tab].replaceChildren(fragment);
  const empty = tab === 'queue' ? state.items.length === 0 : state.history.length === 0;
  $(`#${tab}-empty`).hidden = !empty;
  $(`#${tab}-scroller`).hidden = empty;
  lists[tab].hidden = items.length === 0;
  $(`#${tab}-no-match`).hidden = empty || items.length > 0;
  if (selection[tab] >= 0) select(tab, selection[tab], activeTab === tab);
}

async function reload() {
  state = await invoke('get_state');
  renderList('queue');
  renderList('history');
  if (notesRevision === savedNotesRevision && notes.value !== state.notes) notes.value = state.notes;
}

function listKey(event, tab) {
  if (event.target.closest('button, a')) return;
  const items = visibleItems(tab);
  const item = items[selection[tab]];
  if (!item) return;
  const key = event.key.toLowerCase();
  const ctrl = event.ctrlKey;
  if (event.key === 'Enter' || ctrl && key === 'c') {
    void run(() => invoke('copy_text', { text: item.text, hideAfterCopy: true }));
  } else if (tab === 'queue' && event.key === ' ') {
    void mutate('set_done', { id: item.id, done: !item.done });
  } else if (event.key === 'Delete' || event.key === 'Backspace') {
    void mutate(tab === 'queue' ? 'delete_item' : 'forget_item', { id: item.id }, () => {
      if (!visibleItems(tab).length) (tab === 'queue' ? entry : search).focus();
    });
  } else if (tab === 'history' && ctrl && key === 'z') {
    void mutate('restore_item', { id: item.id });
  } else if (tab === 'queue' && ctrl && ['ArrowUp', 'ArrowDown'].includes(event.key)) {
    // Moving inside a filtered list would jump over hidden items.
    if (queries.queue.trim()) return;
    const delta = event.key === 'ArrowUp' ? -1 : 1;
    void mutate('move_item', { id: item.id, delta }, () => {
      select(tab, state.items.findIndex((candidate) => candidate.id === item.id));
    });
  } else if (event.key === 'ArrowUp') {
    if (selection[tab] === 0) search.focus();
    else select(tab, selection[tab] - 1);
  } else if (event.key === 'ArrowDown') {
    if (tab === 'queue' && selection[tab] === items.length - 1) entry.focus();
    else select(tab, selection[tab] + 1);
  } else return;
  event.preventDefault();
  event.stopPropagation();
}

for (const [tab, list] of Object.entries(lists)) list.addEventListener('keydown', (event) => listKey(event, tab));
menuButton.addEventListener('click', () => openMenu(menu.hidden));
menu.addEventListener('click', (event) => {
  const item = event.target.closest('.menu-item');
  if (!item) return;
  openMenu(false);
  if (item.dataset.view) setTab(item.dataset.view, true);
  else if (item.dataset.theme) {
    localStorage.setItem(THEME_KEY, item.dataset.theme);
    applyTheme();
  }
});
menu.addEventListener('keydown', (event) => {
  if (!['ArrowUp', 'ArrowDown'].includes(event.key)) return;
  event.preventDefault();
  const items = [...menu.querySelectorAll('.menu-item')];
  const index = items.indexOf(document.activeElement) + (event.key === 'ArrowDown' ? 1 : -1);
  items[(index + items.length) % items.length].focus();
});
document.addEventListener('mousedown', (event) => {
  if (!menu.hidden && !event.target.closest('#menu, #menu-button')) openMenu(false);
});
darkScheme.addEventListener('change', applyTheme);
$('#back').addEventListener('click', () => setTab('queue', true));
$('#close').addEventListener('click', () => void run(() => invoke('hide_window')));
$('#clear-done').addEventListener('click', () => void mutate('clear_done'));
$('.composer').addEventListener('mousedown', (event) => {
  if (event.target !== entry) {
    event.preventDefault();
    entry.focus();
  }
});

function fitEntry() {
  entry.style.height = 'auto';
  entry.style.height = `${entry.scrollHeight}px`;
}

entry.addEventListener('input', fitEntry);
entry.addEventListener('keydown', (event) => {
  if (event.key === 'Enter' && !event.shiftKey && !event.isComposing) {
    const text = entry.value;
    event.preventDefault();
    void mutate('add_prompt', { text }, (added) => {
      if (added) {
        if (entry.value === text) {
          entry.value = '';
          fitEntry();
        }
        $('#queue-scroller').scrollTop = $('#queue-scroller').scrollHeight;
      }
    });
  } else if ((event.key === 'ArrowUp' && entry.selectionStart === 0 && entry.selectionEnd === 0 || event.key === 'Tab' && event.shiftKey)
    && visibleItems('queue').length) {
    event.preventDefault();
    select('queue', selection.queue >= 0 ? selection.queue : visibleItems('queue').length - 1);
  }
});
search.addEventListener('input', () => {
  queries[activeTab] = search.value;
  selection[activeTab] = -1;
  renderList(activeTab);
});
search.addEventListener('keydown', (event) => {
  if (['ArrowDown', 'Tab'].includes(event.key) && !event.shiftKey && visibleItems(activeTab).length) {
    event.preventDefault();
    select(activeTab, 0);
  }
});
notes.addEventListener('input', () => {
  notesRevision += 1;
  clearTimeout(notesTimer);
  notesTimer = setTimeout(() => {
    const revision = notesRevision;
    const text = notes.value;
    void run(async () => {
      await invoke('save_notes', { text });
      savedNotesRevision = revision;
      state.notes = text;
    });
  }, 400);
});
document.addEventListener('keydown', (event) => {
  const key = event.key.toLowerCase();
  if (event.key === 'Escape' && !menu.hidden) {
    openMenu(false);
    menuButton.focus();
  } else if (event.key === 'Escape' && document.activeElement === search && search.value) {
    search.value = '';
    search.dispatchEvent(new Event('input'));
  } else if (event.key === 'Escape') void run(() => invoke('hide_window'));
  else if (event.ctrlKey && key === 'f' && activeTab !== 'notes') search.focus();
  else if (event.ctrlKey && ['n', 'l', '1'].includes(key)) setTab('queue', true);
  else if (event.ctrlKey && key === '2') setTab('notes', true);
  else if (event.ctrlKey && key === '3') setTab('history', true);
  else if (event.ctrlKey && event.shiftKey && event.key === 'Backspace') void mutate('clear_done');
  else return;
  event.preventDefault();
  event.stopPropagation();
}, true);

const WHATSAPP_PATH = '<path fill="#25D366" d="M17.472 14.382c-.297-.149-1.758-.867-2.03-.967-.273-.099-.471-.148-.67.15-.197.297-.767.966-.94 1.164-.173.199-.347.223-.644.075-.297-.15-1.255-.463-2.39-1.475-.883-.788-1.48-1.761-1.653-2.059-.173-.297-.018-.458.13-.606.134-.133.298-.347.446-.52.149-.174.198-.298.298-.497.099-.198.05-.371-.025-.52-.075-.149-.669-1.612-.916-2.207-.242-.579-.487-.5-.669-.51-.173-.008-.371-.01-.57-.01-.198 0-.52.074-.792.372-.272.297-1.04 1.016-1.04 2.479 0 1.462 1.065 2.875 1.213 3.074.149.198 2.096 3.2 5.077 4.487.709.306 1.262.489 1.694.625.712.227 1.36.195 1.871.118.571-.085 1.758-.719 2.006-1.413.248-.694.248-1.289.173-1.413-.074-.124-.272-.198-.57-.347m-5.421 7.403h-.004a9.87 9.87 0 01-5.031-1.378l-.361-.214-3.741.982.998-3.648-.235-.374a9.86 9.86 0 01-1.51-5.26c.001-5.45 4.436-9.884 9.888-9.884 2.64 0 5.122 1.03 6.988 2.898a9.825 9.825 0 012.893 6.994c-.003 5.45-4.437 9.884-9.885 9.884m8.413-18.297A11.815 11.815 0 0012.05 0C5.495 0 .16 5.335.157 11.892c0 2.096.547 4.142 1.588 5.945L.057 24l6.305-1.654a11.882 11.882 0 005.683 1.448h.005c6.554 0 11.89-5.335 11.893-11.893a11.821 11.821 0 00-3.48-8.413Z"/>';

applyTheme();
setTab('queue');

if (tauri) {
  await Promise.all([
    tauri.event.listen('state-changed', () => void run(reload)),
    tauri.event.listen('window-shown', () => {
      if (activeTab === 'queue') entry.focus();
      else if (activeTab === 'history') search.focus();
    }),
  ]);
  await run(reload);
  entry.focus();
}
