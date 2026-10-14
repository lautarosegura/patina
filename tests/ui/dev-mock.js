(() => {
  const initial = {
  "items": [
    {
      "id": "a1",
      "kind": "prompt",
      "text": "Revisar el PR de pagos y proponer tests para el caso de cuotas vencidas",
      "source": "",
      "created": "2026-10-13T20:40:00",
      "done": false
    },
    {
      "id": "a2",
      "kind": "capture",
      "text": "Mañana te paso el presupuesto actualizado con los nuevos precios",
      "source": "WhatsApp Business",
      "created": "2026-10-13T20:28:00",
      "done": false,
      "sender": "Ana Gómez",
      "sent": "2026-10-13T20:27",
      "chat": "Equipo Ventas"
    },
    {
      "id": "a3",
      "kind": "capture",
      "text": "Leer la doc en https://example.com/docs y resumirla en 5 puntos",
      "source": "brave-browser",
      "created": "2026-10-13T20:41:00",
      "done": false,
      "url": "https://github.com/foo/bar",
      "title": "foo/bar en GitHub",
      "host": "github.com",
      "favicon": "3aeb002460381c6f258e.png"
    },
    {
      "id": "a4",
      "kind": "capture",
      "text": "Texto copiado desde la terminal",
      "source": "foot",
      "created": "2026-10-13T20:42:00",
      "done": true
    },
    {
      "id": "a5",
      "kind": "capture",
      "text": "Mensaje con la hora original de WhatsApp, sin grupo.",
      "source": "WhatsApp",
      "sender": "María",
      "sent": "8:27 PM, 10/13/2026",
      "created": "2026-10-13T20:43:00",
      "done": false
    },
    {
      "id": "a6",
      "kind": "capture",
      "text": "Mensaje sin hora de envío.",
      "source": "",
      "sender": "Ana",
      "chat": "Equipo",
      "created": "2026-10-13T20:43:00",
      "done": false
    },
    {
      "id": "a7",
      "kind": "capture",
      "text": "Página sin favicon, con dominio y título.",
      "source": "brave-browser",
      "host": "example.com",
      "url": "https://example.com/",
      "title": "Página de ejemplo",
      "created": "2026-10-13T20:43:00",
      "done": false
    },
    {
      "id": "a8",
      "kind": "capture",
      "text": "Dominio sin URL y favicon inexistente.",
      "source": "brave-browser",
      "host": "docs.example.com",
      "favicon": "missing.png",
      "created": "2026-10-13T20:43:00",
      "done": false
    },
    {
      "id": "a9",
      "kind": "prompt",
      "text": "Prompt con origen, saltos de línea y texto largo.\nRevisar comportamiento y mantener el contexto de las capturas. Revisar comportamiento y mantener el contexto de las capturas. Revisar comportamiento y mantener el contexto de las capturas. Revisar comportamiento y mantener el contexto de las capturas. Revisar comportamiento y mantener el contexto de las capturas. Revisar comportamiento y mantener el contexto de las capturas. Revisar comportamiento y mantener el contexto de las capturas. Revisar comportamiento y mantener el contexto de las capturas. Revisar comportamiento y mantener el contexto de las capturas. Revisar comportamiento y mantener el contexto de las capturas. Revisar comportamiento y mantener el contexto de las capturas. Revisar comportamiento y mantener el contexto de las capturas. Revisar comportamiento y mantener el contexto de las capturas. Revisar comportamiento y mantener el contexto de las capturas. ",
      "source": "T3Code",
      "created": "2026-10-13T20:43:00",
      "done": false
    },
    {
      "id": "a10",
      "kind": "capture",
      "text": "Captura sin origen. Texto con <etiquetas> y dos enlaces: https://example.com/a?q=1&b=2), https://example.com/b.",
      "source": "",
      "created": "2026-10-13T20:43:00",
      "done": false
    }
  ],
  "history": [
    {
      "id": "h3",
      "kind": "capture",
      "text": "Mensaje con la hora original de WhatsApp, sin grupo.",
      "source": "WhatsApp",
      "sender": "María",
      "sent": "8:27 PM, 10/13/2026",
      "created": "2026-10-13T20:43:00",
      "done": false,
      "archived": "2026-10-12T09:00:00"
    },
    {
      "id": "h4",
      "kind": "capture",
      "text": "Captura sin origen. Texto con <etiquetas> y dos enlaces: https://example.com/a?q=1&b=2), https://example.com/b.",
      "source": "",
      "created": "2026-10-13T20:43:00",
      "done": false,
      "archived": "2026-10-12T09:30:00"
    },
    {
      "id": "h1",
      "kind": "prompt",
      "text": "Prompt viejo que ya usé",
      "source": "",
      "created": "2026-10-12T10:00:00",
      "done": true,
      "archived": "2026-10-13T09:00:00"
    },
    {
      "id": "h2",
      "kind": "capture",
      "text": "Captura vieja de una página",
      "source": "brave-browser",
      "created": "2026-10-12T11:00:00",
      "done": false,
      "url": "https://example.com/",
      "host": "example.com",
      "archived": "2026-10-13T09:30:00"
    }
  ],
  "notes": "## Ideas\n- probar captura en Cursor\n- revisar atajos\n"
};
  const favicon = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAYAAABzenr0AAADaklEQVR4nLSXS2yNQRTHz62qUkHTekVLlYZKvCJUJUJU0lhUQ4QIC7Gws7BA2ohSz2DlkUgXJCyFSFBEQmNDI94SGvrwKFpENVWv6vU7986Nz9eZ3tv2+if/nrlzZs75f/N9M3OaKDEiGAwmY4rgcjgZphsqPho+hWfhpUAg8D2WuIFoA0g8ErMbroEDJTa0w1NwO0I+SG8EkDgJsw1uhsnSO6iQfUqE/JZYBZA8DXMZzpb44AZchogvEk0AyfX9XoHjJL54Dhcj4pVTgHnfd+EY+T9QEXmI+BzpSPAkH4y56kteLeF3+EB6jsdwv4SXP4IceNHsqH8FSPhLn+4Lchi1pXAm7QL4wuNrhY/gHdji6X8JC5kzDZbQPuSLOQ9ujfwIvQIUZWFqYJJv8AyCPIz8MMpzYYN3GY1vCGY8rMfX6unPxtT64rbBbN2ikRUotyRXeFdI9HCB9/3Jja9VxXqTG/S3xNXXXRJKgEI9XFaKHbnSd0xy9K8jdz99wiVwgGVAE6yUvuMabLD0p8J8FbDUMbGM5WyRPoIY3zClDndRgGW4TSPP5+hQhUxukzjAHOv6MP67pFJXYJRlTlO8kiuI9RNTb3GNTeTPCIsjpqu0h7A9UKYKCFocaRJ/pFv6Qrug2eIYxntLlTjBbPVMi6tZBbxzzFsk8cN8sR9IjSqg2jFpo8QPWxz9t1TATYdzAUvXZxHE2CDhi8yGKj0HUmg0wqGOQWVwj6uk6iaxfuB6AO0Qe+X1CWYkEPgrjRMexzGoql+b3zthLQFL4GiJnjgDai1ZZ+a66s6K0OUWmSThakWvW73NVsF7Ev4+snwTL8BiJgctyc9gVkh0aLE6gRjvQ9ctjTeYg8ap9/p5Y4ul6wFy1JbcYK/Ehl2aPJRb/qrXvarFR47pOsmg9fTn0y40fVoHHKf/hy0qY/VWjXaKPoGzzPHcpSjVSlgvJ70fdMBqBp6THoAYvzCJDreWa3MjT99FgAkwBVMFh5suLS617ot8lOUE6BS3gE6xf3g6fyFz67ydrn9MJmKui/347E+QDnELsH0fz2AB8976HQm2IAzU6lcr5AqxX1au5P7jVs+OI3COLXkol0QPOhWzCa41Yga5DiXGajzdNSrkNDzA2Jru4kcV4AmudUMKAeujjNPSvJ1xTRID/gAAAP//eUeXVQAAAAZJREFUAwDeahm2jKtUkwAAAABJRU5ErkJggg==";
  let state = structuredClone(initial);
  const listeners = new Map();
  const calls = [];
  let nextId = 1;
  const localISO = () => {
    const date = new Date();
    const pad = (value) => String(value).padStart(2, '0');
    return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`;
  };
  const archive = (predicate) => {
    const archived = localISO();
    state.history.push(...state.items.filter(predicate).map((item) => ({ ...item, archived })));
    state.items = state.items.filter((item) => !predicate(item));
  };
  const emit = (name) => {
    for (const callback of listeners.get(name) || []) callback({ event: name, payload: null });
  };
  window.__PATINA_MOCK__ = {
    calls,
    hidden: false,
    clipboard: '',
    openedURLs: [],
    get state() { return structuredClone(state); },
    reset() { state = structuredClone(initial); calls.length = 0; this.hidden = false; this.clipboard = ''; this.openedURLs.length = 0; emit('state-changed'); },
    setState(next) { state = structuredClone(next); emit('state-changed'); },
    emit,
  };
  window.__TAURI__ = {
    core: {
      async invoke(name, args = {}) {
        calls.push({ name, args: structuredClone(args) });
        switch (name) {
          case 'get_state': return structuredClone(state);
          case 'add_prompt': {
            const text = args.text.replace(/^\n+|\n+$/g, '');
            const last = state.items.at(-1);
            if (!text.trim() || last && !last.done && last.text === text) return false;
            state.items.push({ id: `dev-${nextId++}`, kind: 'prompt', text, source: '', created: localISO(), done: false });
            return true;
          }
          case 'set_done': {
            const item = state.items.find((item) => item.id === args.id);
            if (item) item.done = args.done;
            break;
          }
          case 'delete_item': archive((item) => item.id === args.id); break;
          case 'clear_done': archive((item) => item.done); break;
          case 'move_item': {
            const index = state.items.findIndex((item) => item.id === args.id);
            const target = index + args.delta;
            if (index >= 0 && target >= 0 && target < state.items.length) {
              [state.items[index], state.items[target]] = [state.items[target], state.items[index]];
            }
            break;
          }
          case 'restore_item': {
            const item = state.history.find((item) => item.id === args.id);
            if (item) {
              const { archived, ...restored } = item;
              state.history = state.history.filter((candidate) => candidate.id !== args.id);
              state.items.push({ ...restored, done: false });
            }
            break;
          }
          case 'forget_item': state.history = state.history.filter((item) => item.id !== args.id); break;
          case 'copy_text': window.__PATINA_MOCK__.clipboard = args.text; if (args.hideAfterCopy) window.__PATINA_MOCK__.hidden = true; break;
          case 'save_notes': state.notes = args.text; break;
          case 'open_url': window.__PATINA_MOCK__.openedURLs.push(args.url); break;
          case 'hide_window': window.__PATINA_MOCK__.hidden = true; break;
          case 'favicon_url': return args.name === '3aeb002460381c6f258e.png' ? favicon : null;
          default: throw new Error(`Unknown mock command: ${name}`);
        }
        return null;
      },
    },
    event: {
      async listen(name, callback) {
        if (!listeners.has(name)) listeners.set(name, new Set());
        listeners.get(name).add(callback);
        return () => listeners.get(name).delete(callback);
      },
    },
  };
})();
