"""Mouse virtual persistente para Wayland (protocolo zwlr_virtual_pointer_v1), sin dependencias.

Sway sin pantalla no tiene mouse, y las herramientas como wlrctl crean uno que desaparece al
terminar cada comando, así que los clicks no llegan a las apps. Este proceso mantiene un mouse
conectado y lee órdenes por stdin, una por línea:
    move X Y      mueve a la posición absoluta X, Y de la salida
    click [N]     N clicks con el botón izquierdo (1 por defecto)
Uso: python3 -I vpointer.py ANCHO ALTO   (el tamaño de la salida, para las posiciones absolutas)
"""

import os
import socket
import struct
import sys
import time

WIDTH, HEIGHT = int(sys.argv[1]), int(sys.argv[2])
BTN_LEFT = 0x110
WL_DISPLAY = 1
GET_REGISTRY = 1
REGISTRY_GLOBAL = 0
CREATE_VIRTUAL_POINTER = 0
MOTION_ABSOLUTE = 1

sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
sock.connect(os.path.join(os.environ["XDG_RUNTIME_DIR"], os.environ["WAYLAND_DISPLAY"]))
next_id = WL_DISPLAY + 1


def new_id() -> int:
    global next_id
    next_id += 1
    return next_id - 1


def request(obj: int, opcode: int, payload: bytes = b"") -> None:
    sock.sendall(struct.pack("<II", obj, (8 + len(payload)) << 16 | opcode) + payload)


def string(s: str) -> bytes:
    data = s.encode() + b"\0"
    return struct.pack("<I", len(data)) + data + b"\0" * (-len(data) % 4)


buffer = b""


def events():
    global buffer
    sock.settimeout(0.3)
    try:
        while True:
            chunk = sock.recv(65536)
            if not chunk:
                break
            buffer += chunk
    except socket.timeout:
        pass
    while len(buffer) >= 8:
        obj, size_op = struct.unpack("<II", buffer[:8])
        size = size_op >> 16
        if len(buffer) < size:
            break
        yield obj, size_op & 0xFFFF, buffer[8:size]
        buffer = buffer[size:]


registry = new_id()
request(WL_DISPLAY, GET_REGISTRY, struct.pack("<I", registry))
globals_ = {}
for obj, opcode, body in events():
    if obj == registry and opcode == REGISTRY_GLOBAL:
        name, length = struct.unpack("<II", body[:8])
        interface = body[8:8 + length - 1].decode()
        (version,) = struct.unpack("<I", body[8 + length + (-length % 4):][:4])
        globals_[interface] = (name, version)


def bind(interface: str, version: int) -> int:
    obj = new_id()
    name = globals_[interface][0]
    request(registry, 0, struct.pack("<I", name) + string(interface) + struct.pack("<II", version, obj))
    return obj


seat = bind("wl_seat", 1)
manager = bind("zwlr_virtual_pointer_manager_v1", 1)
pointer = new_id()
request(manager, CREATE_VIRTUAL_POINTER, struct.pack("<II", seat, pointer))


def now() -> int:
    return int(time.monotonic() * 1000) & 0xFFFFFFFF


def frame() -> None:
    request(pointer, 4)


for line in sys.stdin:
    parts = line.split()
    if not parts:
        continue
    if parts[0] == "move":
        x, y = int(parts[1]), int(parts[2])
        request(pointer, MOTION_ABSOLUTE, struct.pack("<IIIII", now(), x, y, WIDTH, HEIGHT))
        frame()
    elif parts[0] == "click":
        for _ in range(int(parts[1]) if len(parts) > 1 else 1):
            request(pointer, 2, struct.pack("<III", now(), BTN_LEFT, 1))
            frame()
            time.sleep(0.04)
            request(pointer, 2, struct.pack("<III", now(), BTN_LEFT, 0))
            frame()
            time.sleep(0.08)
    print("ok", flush=True)
    list(events())  # drain, so the compositor never blocks on a full socket
