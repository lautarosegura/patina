"""Genera tests/parity.json ejecutando la versión Python de Patina (la referencia del port).

La referencia es la última versión en Python: git show 79ebd1c:patina > /tmp/patina-py
Uso: python3 -I scripts/parity/gen_fixtures.py /tmp/patina-py tests/parity.json
Los tests de Rust (src-tauri/src/parity_tests.rs) leen el JSON y exigen los mismos resultados.
"""

import json
import os
import struct
import subprocess
import sys
import tempfile
import time
from importlib.machinery import SourceFileLoader

SOURCE, OUT = sys.argv[1], sys.argv[2]
sys.argv = ["patina"]
py = SourceFileLoader("patina_py", SOURCE).load_module()

fixtures: dict[str, list] = {}

# --- parse_sent y format_sent ---
SENT = [
    ("8:27 PM, 10/13/2026", ""), ("8:27 pm, 10/13/2026", "mdy"), ("20:27, 13/10/2026", "dmy"),
    ("20:27, 3/4/2026", "dmy"), ("20:27, 3/4/2026", "mdy"), ("8:05 a. m., 3/4/2026", "dmy"),
    ("12:10 AM, 1/2/2026", "mdy"), ("12:10 PM, 1/2/2026", "mdy"), ("9.15, 2026-04-03", "ymd"),
    ("21:00, 2026/12/31", ""), ("7:00, 31/12/26", ""), ("10:00, 31/02/2026", "dmy"),
    ("basura", ""), ("", "dmy"), ("20:27", "dmy"),
]
fixtures["parse_sent"] = [{"raw": r, "order": o, "expected": py.parse_sent(r, o)} for r, o in SENT]
fixtures["format_sent"] = [
    {"sent": s, "expected": py.format_sent(s)}
    for s in ["2026-10-13T20:27", "2026-01-02T00:10:59", "8:27 PM, 10/13/2026", "texto raro", "20:27, 3/4/2026"]
]
fixtures["format_time"] = [
    {"iso": s, "expected": py.format_time(s)} for s in ["2026-10-13T20:27:00", "2026-01-05T07:03", "2026-12-31T23:59:59"]
]

# --- window_source ---
WINDOWS = [
    {"class": "brave-browser", "title": "WhatsApp - Brave"},
    {"class": "brave-browser", "title": "(19) WhatsApp Business - Brave"},
    {"class": "brave-browser", "title": "Artículo sobre WhatsApp - Brave"},
    {"class": "com.t3tools.T3Code", "title": "T3 Code"},
    {"class": "foot", "title": "~"},
    {"class": "", "title": ""},
    {"title": "(3) WhatsApp - Google Chrome"},
]
fixtures["window_source"] = [{"window": w, "expected": py.window_source(w)} for w in WINDOWS]


# --- linkify, como segmentos (texto o link) ---
def segments(text: str) -> list[dict]:
    out, pos = [], 0
    for m in py.URL_RE.finditer(text):
        url = m.group().rstrip(".,;:!?)]}")
        if text[pos:m.start()]:
            out.append({"text": text[pos:m.start()]})
        out.append({"link": url})
        pos = m.start() + len(url)
    if text[pos:]:
        out.append({"text": text[pos:]})
    return out


LINK_TEXTS = [
    "mirá https://github.com/foo/bar.", "a<b> (ver https://x.com/a?b=1&c=2)", "sin link",
    "http://a.com y https://b.com/x),", "https://ejemplo.com/ruta?q=\"citado\"", "", "https://solo.com",
]
fixtures["linkify"] = [{"text": t, "expected": segments(t)} for t in LINK_TEXTS]


# --- page_text y browser_context: archivos temporales en lugar de $XDG_RUNTIME_DIR ---
tmp = tempfile.mkdtemp()
py.SELECTION_PATH = os.path.join(tmp, "selection.json")
py.PAGE_PATH = os.path.join(tmp, "page.json")
py.FAVICON_DIR = os.path.join(tmp, "favicons")
os.makedirs(py.FAVICON_DIR)
open(os.path.join(py.FAVICON_DIR, py.favicon_name("github.com")), "w").close()
NOW = time.time()


def put(path: str, data: dict | None, age: float | None = None) -> dict | None:
    if data is None:
        if os.path.exists(path):
            os.remove(path)
        return None
    data = dict(data)
    if age is not None:
        data["received"] = NOW - age
    json.dump(data, open(path, "w"))
    return data


WA_SEL = {"text": "Mañana te paso el\npresupuesto", "url": "https://web.whatsapp.com/", "host": "web.whatsapp.com",
          "title": "WhatsApp", "favicon": "wa.png", "messages": [{"time": "8:27 PM, 10/13/2026", "sender": "Juan Pérez"}],
          "chat": "Equipo Ventas", "date_order": "mdy"}
X_SEL = {"text": "here's a prompt you can try asking @bot:\n\ncreate", "url": "https://x.com/p/1", "host": "x.com",
         "title": "lauren on X", "favicon": "x.png", "messages": [], "chat": "", "date_order": ""}
PAGE = {"kind": "page", "tab": "7", "url": "https://www.youtube.com/watch?v=abc", "host": "www.youtube.com",
        "title": "Video - YouTube", "favicon": "yt.png", "since": NOW - 60}
CONTEXT_CASES = [
    ("whatsapp coincide", "Mañana te paso el presupuesto", "WhatsApp Business - Brave", WA_SEL, 5, None),
    ("whatsapp chat individual", "Hola", "WhatsApp - Brave",
     {**WA_SEL, "text": "Hola", "chat": "Ana", "messages": [{"time": "20:27, 13/10/2026", "sender": "Ana"}], "date_order": "dmy"}, 5, None),
    ("whatsapp varios remitentes", "uno dos", "WhatsApp - Brave",
     {**WA_SEL, "text": "uno\ndos", "messages": [{"time": "x", "sender": "A"}, {"time": "y", "sender": "B"}, {"time": "z", "sender": "A"}]}, 5, None),
    ("x con menciones", "here's a prompt you can try asking \n@bot\n:\n\ncreate", "lauren on X - Brave", X_SEL, 5, None),
    ("selección vieja (11 min)", "Mañana te paso el presupuesto", "foot", WA_SEL, 660, None),
    ("texto contenido en la selección", "presupuesto", "foot", WA_SEL, 5, None),
    ("barra de direcciones", "https://www.youtube.com/watch?v=abc", "Video - YouTube - Brave", X_SEL, 5, PAGE),
    ("pestaña de otra ventana", "https://www.youtube.com/watch?v=abc", "Otra - Brave", X_SEL, 5, PAGE),
    ("url con favicon guardado", "https://github.com/foo", "foot", None, None, None),
    ("url sin favicon", "https://example.com/x", "foot", None, None, None),
    ("texto común", "hola mundo", "foot", None, None, None),
    ("texto vacío", "   ", "foot", X_SEL, 5, PAGE),
]
fixtures["browser_context"] = []
fixtures["page_text"] = []
for name, text, title, sel, age, page in CONTEXT_CASES:
    s = put(py.SELECTION_PATH, sel, age)
    p = put(py.PAGE_PATH, page)
    fixtures["browser_context"].append({"name": name, "text": text, "window_title": title, "selection": s, "page": p,
                                         "favicons": ["github.com"], "now": NOW, "expected": py.browser_context(text, title)})
    fixtures["page_text"].append({"name": name, "text": text, "selection": s, "now": NOW, "expected": py.page_text(text)})


# --- host nativo: el binario real por stdin/stdout, con sus archivos en un directorio temporal ---
def host_run(messages: list[bytes], runtime: str, data: str) -> tuple[list[dict], dict, dict, list[str]]:
    payload = b"".join(struct.pack("<I", len(m)) + m for m in messages)
    env = {"XDG_RUNTIME_DIR": runtime, "XDG_DATA_HOME": data, "PATH": "/usr/bin"}
    out = subprocess.run([sys.executable, SOURCE, "chrome-extension://fedemognlmdojgmeinfhhjliccpfncdo/"],
                         input=payload, capture_output=True, env=env, check=True).stdout
    replies = []
    while out:
        (n,) = struct.unpack("<I", out[:4])
        replies.append(json.loads(out[4:4 + n]))
        out = out[4 + n:]

    def load(name: str) -> dict:
        try:
            return json.load(open(os.path.join(runtime, name)))
        except OSError:
            return {}

    fav_dir = os.path.join(data, "patina", "favicons")
    return replies, load("patina-selection.json"), load("patina-page.json"), sorted(os.listdir(fav_dir)) if os.path.isdir(fav_dir) else []


PNG = b"\x89PNG\r\n\x1a\n" + b"\x00" * 16
import base64  # noqa: E402

b64 = lambda b: base64.b64encode(b).decode()  # noqa: E731
HOST_CASES = [
    ("selección web con favicon", [{"text": "hola", "url": "https://github.com/a", "title": "A", "favicon": b64(PNG)}]),
    ("favicon que no es png", [{"text": "hola", "url": "https://evil.example/", "favicon": b64(b"<svg/>")}]),
    ("favicon base64 inválido", [{"text": "hola", "url": "https://evil.example/", "favicon": "@@@"}]),
    ("whatsapp", [{"text": "msg", "url": "https://web.whatsapp.com/", "title": "WhatsApp",
                   "messages": [{"time": "20:27, 13/10/2026", "sender": "Ana"}], "chat": "Grupo", "dateOrder": "dmy"}]),
    ("orden de fecha inválido", [{"text": "msg", "url": "https://web.whatsapp.com/", "messages": [], "dateOrder": "xyz"}]),
    ("selección vacía", [{"text": "", "url": "https://x.com/", "title": "X"}]),
    ("sin text", [{"url": "https://x.com/"}]),
    ("json roto", [b"{no es json"]),
    ("página", [{"kind": "page", "tab": 7, "url": "https://youtube.com/w", "title": "Video"}]),
    ("página: cambia el título", [{"kind": "page", "tab": 7, "url": "https://youtube.com/w", "title": "Video"},
                                  {"kind": "page", "tab": 7, "url": "https://youtube.com/w", "title": "(1) Video"}]),
    ("página: cambia la pestaña", [{"kind": "page", "tab": 7, "url": "https://youtube.com/w", "title": "Video"},
                                   {"kind": "page", "tab": 8, "url": "https://youtube.com/w", "title": "Video"}]),
]
fixtures["native_host"] = []
for name, msgs in HOST_CASES:
    runtime, data = tempfile.mkdtemp(), tempfile.mkdtemp()
    raw = [m if isinstance(m, bytes) else json.dumps(m).encode() for m in msgs]
    replies = []
    for i, message in enumerate(raw):  # un proceso por mensaje, como sendNativeMessage del navegador
        if i:
            time.sleep(0.05)
        t_last = time.time()
        r, sel, page, favs = host_run([message], runtime, data)
        replies += r
    for record in (sel, page):  # "now": lo fijó el último mensaje; "kept": viene de uno anterior
        for key in ("received", "since"):
            if key in record:
                record[key] = "now" if record[key] >= t_last else "kept"
    fixtures["native_host"].append({"name": name, "messages": [r.decode("utf-8", "replace") for r in raw],
                                    "replies": replies, "selection": sel, "page": page, "favicons": favs})


# --- Store: secuencias de operaciones ---
class FakeStore(py.Store):
    def __init__(self) -> None:
        self.items, self.history = [], []

    def save(self) -> None: ...

    def save_history(self) -> None: ...


def normalize(items: list[dict]) -> list[dict]:
    return [{k: ("<ts>" if k in ("created", "archived") else v) for k, v in i.items() if k != "id"} for i in items]


store_ops = []
st = FakeStore()
log = []
for op in [
    ("add", "prompt", "uno", "", None), ("add", "prompt", "uno", "", None), ("add", "capture", "\n\ndos\n", "foot", {"host": "x.com"}),
    ("add", "prompt", "   ", "", None), ("done", 0), ("add", "prompt", "uno", "", None), ("archive_done",), ("restore", 0),
    ("forget_first",), ("add", "prompt", "tres", "", None), ("delete", 0),
]:
    if op[0] == "add":
        ok = st.add(op[1], op[2], op[3], op[4])
    elif op[0] == "done":
        st.items[op[1]]["done"] = True
        ok = True
    elif op[0] == "archive_done":
        st.archive([i for i in st.items if i["done"]])
        ok = True
    elif op[0] == "restore":
        st.restore(st.history[op[1]]["id"]) if st.history else None
        ok = True
    elif op[0] == "forget_first":
        st.forget(st.history[0]["id"]) if st.history else None
        ok = True
    elif op[0] == "delete":
        st.archive([st.items[op[1]]])
        ok = True
    log.append({"op": list(op), "result": ok, "items": normalize(st.items), "history": normalize(st.history)})
fixtures["store"] = log


# --- decisión de captura (PatinaApp.capture con el sistema simulado) ---
class Notifs(list):
    pass


def capture_case(primary: str, changed_ago: float | None, captured_ago: float | None, title: str,
                 page: dict | None, selection: dict | None) -> dict:
    notes = Notifs()
    store = FakeStore()
    app = type("App", (), {})()
    app.store, app.window = store, None
    now = time.time()
    app.primary_changed_at = 0.0 if changed_ago is None else now - changed_ago
    app.last_capture_at = 0.0 if captured_ago is None else now - captured_ago
    put(py.PAGE_PATH, page)
    put(py.SELECTION_PATH, selection)
    py.run = lambda cmd, stdin=None: primary
    py.active_window = lambda: {"class": "brave-browser" if title.endswith(" - Brave") else "foot", "title": title}
    py.notify = lambda summary, body="": notes.append(summary)
    py.PatinaApp.capture(app)
    item = store.items[-1] if store.items else None
    return {"now": now,
            "item": {k: v for k, v in item.items() if k in ("text", "source", "url", "host", "title")} if item else None,
            "notified": list(notes), "consumed": app.last_capture_at > now - 1}


PAGE_RECENT = {**PAGE, "since": time.time() - 2, "received": time.time()}
PAGE_OLD = {**PAGE, "since": time.time() - 600, "received": time.time()}
EMPTY_SEL = {"text": "", "url": PAGE["url"], "received": time.time()}
CAPTURE_CASES = [
    ("selección nueva fuera de Brave", "texto A", 1, None, "foot", None, None),
    ("ya capturada", "texto A", 5, 2, "foot", None, None),
    ("vencida (más de 10 s)", "texto B", 15, None, "foot", None, None),
    ("sin vigilante: se acepta", "texto C", None, None, "foot", None, None),
    ("vacía fuera de Brave", "", 1, None, "foot", None, None),
    ("Brave: texto nuevo en la página", "texto D", 1, None, "Video - YouTube - Brave", PAGE_OLD, None),
    ("Brave: selección de antes de la pestaña", "texto viejo", 5, None, "Video - YouTube - Brave", PAGE_RECENT, None),
    ("Brave: deseleccionó en la página", "texto E", 3, None, "Video - YouTube - Brave", PAGE_OLD, {**EMPTY_SEL, "received": time.time() - 1}),
    ("Brave: nada seleccionado", "", 1, None, "Video - YouTube - Brave", PAGE_OLD, None),
    ("Brave: otra ventana con otro título", "texto F", 1, None, "Otra cosa - Brave", PAGE_OLD, None),
]
fixtures["capture"] = [
    {"name": name, "primary": primary, "changed_ago": c, "captured_ago": k, "window_title": title,
     "page": page, "selection": sel, "expected": capture_case(primary, c, k, title, page, sel)}
    for name, primary, c, k, title, page, sel in CAPTURE_CASES
]

json.dump(fixtures, open(OUT, "w"), ensure_ascii=False, indent=1)
print(f"{OUT}: " + ", ".join(f"{k}={len(v)}" for k, v in fixtures.items()))
