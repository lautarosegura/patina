"""Compara el host nativo de dos ejecutables (Python de referencia y el port) por stdin/stdout.

Uso: python3 -I scripts/parity/host_binary.py <patina de referencia> <patina nuevo>
La referencia Python sale de: git show 79ebd1c:patina > /tmp/patina-py && chmod +x /tmp/patina-py
Cada caso corre un proceso por mensaje, como el navegador. Imprime una línea por caso y sale
con 1 si alguno difiere.
"""

import base64
import json
import os
import stat
import struct
import subprocess
import sys
import tempfile
import time

REFERENCE, CANDIDATE = sys.argv[1], sys.argv[2]
PNG = b"\x89PNG\r\n\x1a\n" + b"\x00" * 16
B64 = base64.b64encode(PNG).decode()
CASES = {
    "selección con favicon": [{"text": "hola", "url": "https://github.com/a", "title": "A", "favicon": B64}],
    "favicon falso": [{"text": "hola", "url": "https://evil.example/", "favicon": base64.b64encode(b"<svg/>").decode()}],
    "whatsapp": [{"text": "msg", "url": "https://web.whatsapp.com/", "messages": [{"time": "20:27, 13/10/2026", "sender": "Ana"}],
                  "chat": "Grupo", "dateOrder": "dmy"}],
    "selección vacía": [{"text": "", "url": "https://x.com/", "title": "X"}],
    "json roto": [b"{roto"],
    "sin text": [{"url": "https://x.com/"}],
    "pestaña: cambia el título": [{"kind": "page", "tab": 7, "url": "https://y.com/w", "title": "V"},
                                  {"kind": "page", "tab": 7, "url": "https://y.com/w", "title": "(1) V"}],
    "pestaña: cambia la URL": [{"kind": "page", "tab": 7, "url": "https://y.com/w", "title": "V"},
                               {"kind": "page", "tab": 7, "url": "https://y.com/z", "title": "V"}],
}


def run(exe: str, messages: list) -> dict:
    runtime, data = tempfile.mkdtemp(), tempfile.mkdtemp()
    env = {"XDG_RUNTIME_DIR": runtime, "XDG_DATA_HOME": data, "PATH": "/usr/bin", "HOME": data}
    replies, starts = [], []
    for m in messages:
        raw = m if isinstance(m, bytes) else json.dumps(m).encode()
        starts.append(time.time())
        out = subprocess.run([exe, "chrome-extension://fedemognlmdojgmeinfhhjliccpfncdo/"],
                             input=struct.pack("<I", len(raw)) + raw, capture_output=True, env=env, timeout=10).stdout
        while out:
            (n,) = struct.unpack("<I", out[:4])
            replies.append(json.loads(out[4:4 + n]))
            out = out[4 + n:]
        time.sleep(0.05)
    files = {}
    for name in ("patina-selection.json", "patina-page.json"):
        path = os.path.join(runtime, name)
        if os.path.exists(path):
            record = json.load(open(path))
            for key in ("received", "since"):
                if key in record:
                    record[key] = "último" if record[key] >= starts[-1] else "anterior"
            files[name] = {"record": record, "mode": oct(stat.S_IMODE(os.stat(path).st_mode))}
    fav = os.path.join(data, "patina", "favicons")
    return {"replies": replies, "files": files, "favicons": sorted(os.listdir(fav)) if os.path.isdir(fav) else []}


failed = False
for name, messages in CASES.items():
    ref, new = run(REFERENCE, messages), run(CANDIDATE, messages)
    same = ref == new
    failed |= not same
    print(f"{'OK  ' if same else 'DIFF'} {name}")
    if not same:
        print("  referencia:", json.dumps(ref, ensure_ascii=False))
        print("  nuevo:     ", json.dumps(new, ensure_ascii=False))
sys.exit(1 if failed else 0)
