# Patina

Cola de prompts, capturas y bloc de notas para trabajar con IA en Linux con Hyprland.

Resaltás texto en cualquier app, tocás **Shift dos veces** y queda guardado. Escribís tus próximos prompts, los copiás cuando los necesitás y los tachás a medida que avanzás. Todo queda en tu máquina: no sincroniza nada, no recolecta nada y no usa cuentas.

Patina está inspirado en [Copper](https://shadcn.com/copper), la app para macOS de shadcn. Es un proyecto independiente, sin relación con Copper ni con su autor. Si usás Mac, comprá Copper.

## Qué hace

- **Captura con doble Shift**: guarda el texto resaltado (la selección primaria de Wayland) junto con la app de origen y la hora. Solo toma selecciones nuevas: no repite lo que ya capturaste ni lo que resaltaste hace más de 10 segundos.
- **Cola**: escribís prompts y los copiás con Enter. Los links se abren con un click y cada item tiene un botón "Copiar".
- **Notas**: un bloc de texto libre que se guarda solo.
- **Historial**: lo que limpiás o borrás de la cola queda guardado, con búsqueda, y se puede devolver a la cola.
- **Con la extensión del navegador** (Brave, Chrome o Chromium):
  - Cada captura guarda la página: favicon, dominio, título y link para volver a ella.
  - Si no hay una selección nueva, se captura la URL de la pestaña. Así funciona también con la barra de direcciones.
  - En WhatsApp Web guarda quién envió el mensaje, el grupo y la hora de envío.
  - El texto se guarda como se ve en pantalla: Chromium corta la línea alrededor de elementos como las menciones de X, y Patina lo corrige.

## Requisitos

- Hyprland. Probado con 0.56 y configuración en Lua.
- Python 3 con GTK 4 y libadwaita.
- `wl-clipboard`, `libnotify` y `hyprctl`.

En Fedora:

```sh
sudo dnf install python3-gobject gtk4 libadwaita wl-clipboard libnotify
```

## Instalación

```sh
git clone https://github.com/lautarosegura/patina
cd patina
./install.sh
```

`install.sh` crea `~/.local/bin/patina` como enlace al archivo del repo y registra el host nativo en los navegadores Chromium que encuentre. La extensión y la configuración de Hyprland se usan directamente desde el clon, así que conservalo: si lo movés, volvé a correr `install.sh` y actualizá las rutas. No usa `sudo` ni modifica tu configuración.

### Hyprland

Agregá al final de tu configuración la ruta a tu clon (`install.sh` te la muestra) y recargá con `hyprctl reload`:

```lua
-- ~/.config/hypr/hyprland.lua
dofile(os.getenv("HOME") .. "/code/patina/hypr/patina.lua")
```

```ini
# ~/.config/hypr/hyprland.conf (sin probar)
source = ~/code/patina/hypr/patina.conf
```

Si instalaste el comando en otra carpeta con `BIN_DIR=... ./install.sh`, definí `PATINA_BIN = "/ruta/patina"` antes del `dofile`.

Eso agrega el doble Shift, **Super+C** para mostrar u ocultar Patina, la regla de ventana flotante y el arranque al iniciar sesión.

### Extensión del navegador (opcional)

1. Abrí `brave://extensions` o `chrome://extensions`.
2. Activá el **modo de desarrollador**.
3. Tocá **Cargar extensión sin empaquetar** y elegí la carpeta `extension/` de tu clon.
4. Recargá las pestañas que ya tenías abiertas.

La extensión pide estos permisos:

- **Leer los sitios**: para saber de qué página es el texto que resaltás.
- **Pestañas**: el navegador lo muestra como "leer tu historial de navegación". Sirve para conocer la URL y el título de la pestaña activa.
- **Favicons**: para leer los íconos que el navegador ya tiene en caché, sin pedirlos a la red.
- **Native messaging**: para pasarle los datos a Patina.

## Uso

| Acción | Atajo |
| --- | --- |
| Capturar el texto seleccionado | Shift dos veces (menos de 350 ms) |
| Mostrar u ocultar Patina | Super+C |
| Agregar un prompt | Escribir en el campo y Enter |
| Ir de la entrada a la lista | ↓ o Tab |
| Copiar y volver a la app anterior | Enter, Ctrl+C o doble click sobre un item |
| Solo copiar | Botón "Copiar" que aparece al pasar el mouse |
| Tachar o destachar | Checkbox o Espacio |
| Abrir un link o la página de origen | Click sobre el link o el dominio |
| Borrar (pasa al historial) | Supr o Retroceso |
| Reordenar | Ctrl+↑ / Ctrl+↓ |
| Limpiar los completados (pasan al historial) | Ctrl+Shift+Retroceso |
| Cola / Notas / Historial | Ctrl+1 / Ctrl+2 / Ctrl+3 |
| Historial: devolver a la cola | Ctrl+Z o el botón ↶ |
| Historial: borrar para siempre | Supr |
| Ocultar | Esc |

Desde la terminal, `patina add "texto"` agrega un prompt y `patina capture` captura la selección actual.

## Datos

- `~/.local/share/patina/`: `items.json` (la cola), `historial.json`, `notas.md` y `favicons/`.
- `$XDG_RUNTIME_DIR/patina-*.json`: la última selección y la pestaña activa que mandó la extensión. Son archivos en memoria, con permisos `0600`, que se borran al cerrar la sesión.

## Desarrollo

Como el comando, la extensión y la configuración apuntan al clon, los cambios se usan directamente:

- **`patina`:** reiniciá el servicio con `pkill -f "patina --service"; patina --service &`.
- **`extension/`:** recargala en `brave://extensions`.
- **`hypr/`:** corré `hyprctl reload`.

## Cómo funciona

- **Un solo programa:** `patina` es un único archivo Python con GTK 4 y libadwaita. Queda corriendo oculto (`patina --service`), y los demás comandos le llegan por D-Bus.
- **Doble Shift:** Hyprland ejecuta `patina tap` al soltar Shift, sin consumir la tecla. `tap` no carga GTK. Si dos taps llegan con menos de 350 ms de diferencia, se ejecuta `patina capture`.
- **Selecciones viejas:** en Wayland la selección primaria sigue disponible aunque en pantalla ya no haya nada seleccionado. Patina vigila sus cambios con `wl-paste --primary --watch` para descartar las selecciones viejas.
- **Extensión:** `content.js` manda el texto resaltado y, en WhatsApp Web, el remitente, que WhatsApp marca con `data-pre-plain-text`. `background.js` agrega el favicon y la pestaña activa, y lo reenvía por native messaging al host `io.github.lautarosegura.patina`.
- **Host nativo:** es el mismo `patina`, que el navegador ejecuta con `chrome-extension://…`. Guarda los datos en `$XDG_RUNTIME_DIR`, y al capturar Patina los usa solo si coinciden con el texto capturado.
- **ID de la extensión:** el `manifest.json` incluye una clave pública. Así el ID de la extensión (`fedemognlmdojgmeinfhhjliccpfncdo`) es fijo y coincide con el que autoriza el host nativo.

## Desinstalar

```sh
./uninstall.sh          # quita el comando y el host nativo; conserva tus datos
./uninstall.sh --purge  # borra también ~/.local/share/patina
```

## Licencia

[MIT](LICENSE). El logo de WhatsApp es de [Simple Icons](https://simpleicons.org) (CC0).
