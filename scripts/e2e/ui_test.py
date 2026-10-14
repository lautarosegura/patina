import json
import os
import subprocess
import sys
import time

BIN = os.path.abspath(sys.argv[1])
DATA = os.path.join(os.environ["XDG_DATA_HOME"], "patina")
OUT = "out"
failures = []

ITEMS = [
    {"id": "a1", "kind": "prompt", "text": "Revisar el PR de pagos y proponer tests", "source": "",
     "created": "2026-10-13T20:40:00", "done": False},
    {"id": "a2", "kind": "capture", "text": "Mañana te paso el presupuesto", "source": "WhatsApp Business",
     "created": "2026-10-13T20:28:00", "done": False, "sender": "Ana Gómez", "sent": "2026-10-13T20:27",
     "chat": "Equipo Ventas"},
    {"id": "a3", "kind": "capture", "text": "Leer la doc en https://example.com/docs y resumirla", "source": "brave-browser",
     "created": "2026-10-13T20:41:00", "done": False, "url": "https://github.com/foo/bar", "title": "foo/bar en GitHub",
     "host": "github.com", "favicon": "3aeb002460381c6f258e.png"},
    {"id": "a4", "kind": "capture", "text": "Texto copiado desde la terminal", "source": "foot",
     "created": "2026-10-13T20:42:00", "done": True},
]
HISTORY = [{"id": "h1", "kind": "prompt", "text": "Prompt viejo que ya usé", "source": "", "created": "2026-10-12T10:00:00",
            "done": True, "archived": "2026-10-13T09:00:00"}]


def sh(*cmd, input=None):
    return subprocess.run(cmd, input=input, capture_output=True, text=True, timeout=20).stdout


def copy(text, primary=False):
    # wl-copy stays in the background serving the clipboard: no pipes to wait on.
    subprocess.run(["wl-copy", *(["--primary"] if primary else [])], input=text, text=True,
                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def check(name, ok, detail=""):
    print(f"{'OK  ' if ok else 'FAIL'} {name}{'' if ok else '  ' + detail}", flush=True)
    if not ok:
        failures.append(name)


def items():
    return json.load(open(os.path.join(DATA, "items.json")))


def history():
    return json.load(open(os.path.join(DATA, "historial.json")))


def patina_window():
    def walk(node):
        if node.get("app_id") == "patina":
            return node
        for child in node.get("nodes", []) + node.get("floating_nodes", []):
            found = walk(child)
            if found:
                return found
    node = walk(json.loads(sh("swaymsg", "-t", "get_tree")))
    return node if node and node.get("visible") else None


def wait_for(predicate, timeout=4.0):
    end = time.time() + timeout
    while time.time() < end:
        if predicate():
            return True
        time.sleep(0.1)
    return False


# Headless Sway has no input devices: a persistent virtual mouse (vpointer.py) and a wtype that
# stays connected keep both capabilities on the seat.
mouse = subprocess.Popen([sys.executable, "-I", "vpointer.py", "1280", "800"], stdin=subprocess.PIPE,
                         stdout=subprocess.PIPE, text=True)
keyboard = subprocess.Popen(["wtype", "-s", "3600000", "-k", "F24"])


def pointer(command):
    mouse.stdin.write(command + "\n")
    mouse.stdin.flush()
    mouse.stdout.readline()


def hover(x, y):
    r = patina_window()["rect"]
    pointer(f"move {r['x'] + x} {r['y'] + y}")
    time.sleep(0.5)


def click(x, y, count=1):
    hover(x, y)
    pointer(f"click {count}")
    time.sleep(0.4)


def keys(*args):
    sh("wtype", *args)
    time.sleep(0.4)


def shot(name):
    r = patina_window()["rect"]
    sh("grim", "-g", f"{r['x']},{r['y']} {r['width']}x{r['height']}", f"{OUT}/{name}.png")


def red_png(size):
    import struct
    import zlib
    rows = b"".join(b"\0" + b"\xff\x00\x00\xff" * size for _ in range(size))
    chunk = lambda kind, data: struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))
    header = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header) + chunk(b"IDAT", zlib.compress(rows)) + chunk(b"IEND", b"")


def clipboard():
    return sh("wl-paste", "--no-newline")


os.makedirs(DATA, exist_ok=True)
json.dump(ITEMS, open(os.path.join(DATA, "items.json"), "w"), ensure_ascii=False)
json.dump(HISTORY, open(os.path.join(DATA, "historial.json"), "w"), ensure_ascii=False)
open(os.path.join(DATA, "notas.md"), "w").write("## Ideas\n")
os.makedirs(os.path.join(DATA, "favicons"), exist_ok=True)
open(os.path.join(DATA, "favicons", "3aeb002460381c6f258e.png"), "wb").write(red_png(16))

# The app opens links through xdg-open; a fake one records the URL instead of starting a browser.
FAKE_BIN = os.path.abspath(f"{OUT}/fakebin")
os.makedirs(FAKE_BIN, exist_ok=True)
OPENED = os.path.abspath(f"{OUT}/opened.txt")
with open(f"{FAKE_BIN}/xdg-open", "w") as f:
    f.write(f'#!/bin/sh\nprintf "%s\\n" "$1" >> {OPENED}\n')
os.chmod(f"{FAKE_BIN}/xdg-open", 0o755)
service_env = {**os.environ, "PATH": f"{FAKE_BIN}:{os.environ['PATH']}"}
service = subprocess.Popen([BIN, "--service"], stdout=open(f"{OUT}/patina.log", "w"), stderr=subprocess.STDOUT, env=service_env)
time.sleep(2)
subprocess.run([BIN])
check("patina (lo que ejecuta Super+C) muestra la ventana flotante 560x680", wait_for(lambda: patina_window() is not None)
      and patina_window()["rect"]["width"] == 560 and patina_window()["rect"]["height"] == 680, str(patina_window() and patina_window()["rect"]))
time.sleep(1.5)
shot("01-cola")

keys("nuevo prompt desde la UI")
keys("-k", "Return")
check("al mostrarse el foco está en la entrada y Enter agrega el prompt",
      wait_for(lambda: items()[-1]["text"] == "nuevo prompt desde la UI"), items()[-1]["text"])

click(37, 128)
check("click en el checkbox tacha sin cerrar", wait_for(lambda: items()[0]["done"]) and patina_window() is not None)

copy("marcador")
hover(300, 207)
shot("02-hover-fila")
click(491, 195)
time.sleep(0.15)
shot("03-copiado")
check("botón Copiar copia sin tachar ni cerrar", clipboard() == ITEMS[1]["text"] and not items()[1]["done"]
      and patina_window() is not None, repr(clipboard()))

copy("marcador")
click(233, 251)
check("click en un link lo abre sin seleccionar ni copiar",
      wait_for(lambda: os.path.exists(OPENED) and open(OPENED).read().strip() == "https://example.com/docs")
      and clipboard() == "marcador" and patina_window() is not None, repr(open(OPENED).read() if os.path.exists(OPENED) else ""))

click(100, 290)
check("un click en el texto solo selecciona", clipboard() == "marcador" and patina_window() is not None, repr(clipboard()))

click(100, 290, count=2)
check("doble click copia y oculta, sin tachar", wait_for(lambda: patina_window() is None) and clipboard() == ITEMS[2]["text"]
      and not items()[2]["done"], repr(clipboard()))

subprocess.run([BIN])
wait_for(lambda: patina_window() is not None)
time.sleep(0.8)
keys("-M", "ctrl", "-k", "3", "-m", "ctrl")
shot("04-historial")
before = len(history())
keys("-k", "Down")
keys("-M", "ctrl", "-k", "z", "-m", "ctrl")
check("Ctrl+Z en el historial devuelve el item a la cola",
      wait_for(lambda: len(history()) == before - 1 and items()[-1]["text"] == HISTORY[0]["text"]), str(len(history())))

keys("-M", "ctrl", "-k", "2", "-m", "ctrl")
keys("nota escrita en la UI")
check("las notas se guardan solas", wait_for(lambda: open(os.path.join(DATA, "notas.md")).read().endswith("nota escrita en la UI")))
shot("05-notas")

keys("-M", "ctrl", "-k", "1", "-m", "ctrl")
n = len(items())
click(150, 314)
keys("-k", "Delete")
check("Supr manda el item seleccionado al historial", wait_for(lambda: len(items()) == n - 1)
      and history()[-1]["text"] == ITEMS[3]["text"], str(len(items())))

copy("texto capturado desde afuera", primary=True)
time.sleep(0.6)
subprocess.run([BIN, "capture"])
check("una captura externa aparece en la cola", wait_for(lambda: items()[-1]["text"] == "texto capturado desde afuera"))
big = "selección grande " * 12000
copy(big, primary=True)
time.sleep(0.6)
subprocess.run([BIN, "capture"])
check("una selección de ~200 KB se captura entera", wait_for(lambda: len(items()[-1]["text"]) == len(big.strip("\n")), timeout=6),
      str(len(items()[-1]["text"])))
time.sleep(0.8)
time.sleep(0.8)
shot("06-captura-externa")

keys("-k", "Escape")
check("Esc oculta la ventana", wait_for(lambda: patina_window() is None))
check("el servicio sigue vivo después de ocultar", service.poll() is None)

service.terminate()
mouse.terminate()
keyboard.terminate()
print(f"\n{len(failures)} fallas" if failures else "\ntodo OK")
sys.exit(1 if failures else 0)
