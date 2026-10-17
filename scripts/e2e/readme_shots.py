"""Capturas del README en docs/, con datos de ejemplo, en el mismo Sway aislado que ui_test.py."""

import json
import os
import shutil
import subprocess
import sys
import time
from datetime import datetime

BIN = os.path.abspath(sys.argv[1])
DATA = os.path.join(os.environ["XDG_DATA_HOME"], "patina")
DOCS = os.path.abspath("../../docs")
TODAY = datetime.now().strftime("%Y-%m-%d")
GITHUB_FAVICON = "3aeb002460381c6f258e.png"

ITEMS = [
    {"id": "s1", "kind": "prompt", "text": "Refactor the auth middleware to use the new session API, then add tests for expired tokens",
     "source": "", "created": f"{TODAY}T09:12:00", "done": True},
    {"id": "s2", "kind": "capture", "text": "Can you send me the updated pricing before Friday's call?", "source": "WhatsApp",
     "created": f"{TODAY}T10:43:00", "done": False, "sender": "Ana Gómez", "sent": f"{TODAY}T10:42", "chat": "Sales team"},
    {"id": "s3", "kind": "capture", "text": "Build with --release for production: debug builds skip LTO and are 3x bigger.",
     "html": "Build with <code>--release</code> for production: <b>debug builds skip LTO</b> and are 3x bigger.",
     "source": "brave-browser", "created": f"{TODAY}T11:05:00", "done": False, "url": "https://github.com/tauri-apps/tauri",
     "title": "tauri-apps/tauri", "host": "github.com", "favicon": GITHUB_FAVICON},
    {"id": "s4", "kind": "prompt", "text": "Summarize https://tauri.app/start/ in five bullet points for the onboarding doc",
     "source": "", "created": f"{TODAY}T11:20:00", "done": False},
    {"id": "s5", "kind": "capture", "text": "error[E0502]: cannot borrow `store` as mutable because it is also borrowed as immutable",
     "source": "foot", "created": f"{TODAY}T11:31:00", "done": False},
]
HISTORY = [
    {"id": "h1", "kind": "prompt", "text": "Write the release notes for 0.2", "source": "", "created": f"{TODAY}T08:00:00",
     "done": True, "archived": f"{TODAY}T08:30:00"},
    {"id": "h2", "kind": "capture", "text": "Native messaging hosts read a 4-byte length prefix in native byte order",
     "source": "brave-browser", "created": f"{TODAY}T08:10:00", "done": False, "url": "https://github.com/tauri-apps/tauri",
     "host": "github.com", "favicon": GITHUB_FAVICON, "archived": f"{TODAY}T08:31:00"},
]


def sh(*cmd):
    return subprocess.run(cmd, capture_output=True, text=True, timeout=20).stdout


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


def wait_for(predicate, timeout=5.0):
    end = time.time() + timeout
    while time.time() < end and not predicate():
        time.sleep(0.1)


mouse = subprocess.Popen([sys.executable, "-I", "vpointer.py", "1280", "800"], stdin=subprocess.PIPE,
                         stdout=subprocess.PIPE, text=True)
keyboard = subprocess.Popen(["wtype", "-s", "3600000", "-k", "F24"])


def pointer(command):
    mouse.stdin.write(command + "\n")
    mouse.stdin.flush()
    mouse.stdout.readline()


def at(x, y, clicks=0):
    r = patina_window()["rect"]
    pointer(f"move {r['x'] + x} {r['y'] + y}")
    if clicks:
        pointer(f"click {clicks}")
    time.sleep(0.5)


def shot(name):
    r = patina_window()["rect"]
    sh("grim", "-g", f"{r['x']},{r['y']} {r['width']}x{r['height']}", f"{DOCS}/{name}.png")


os.makedirs(os.path.join(DATA, "favicons"), exist_ok=True)
shutil.copy("github-favicon.png", os.path.join(DATA, "favicons", GITHUB_FAVICON))
json.dump(ITEMS, open(os.path.join(DATA, "items.json"), "w"), ensure_ascii=False)
json.dump(HISTORY, open(os.path.join(DATA, "historial.json"), "w"), ensure_ascii=False)
open(os.path.join(DATA, "notas.md"), "w").write("")

service = subprocess.Popen([BIN, "--service"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
time.sleep(2)
subprocess.run([BIN])
wait_for(lambda: patina_window() is not None)
time.sleep(1.5)
at(300, 170)
at(489, 170, clicks=1)
time.sleep(0.15)
shot("screenshot-queue")
time.sleep(2)
sh("wtype", "-M", "ctrl", "-k", "3", "-m", "ctrl")
time.sleep(0.8)
shot("screenshot-history")
service.terminate()
mouse.terminate()
keyboard.terminate()
print("capturas en docs/")
