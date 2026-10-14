# Patina

A prompt queue, capture inbox and scratchpad for AI-assisted work on Linux with Hyprland.

Highlight text in any app, tap **Shift twice**, and it's saved. Write your upcoming prompts, copy them when you need them, and check them off as you go. Everything stays on your machine: no sync, no telemetry, no account.

Patina is inspired by [Copper](https://shadcn.com/copper), shadcn's macOS app. It is an independent project with no affiliation with Copper or its author. If you're on a Mac, buy Copper.

> The interface is currently in Spanish.

## Features

- **Double Shift capture**: saves the highlighted text (the Wayland primary selection) along with the source app and time. Only fresh selections count: it won't re-capture what you already captured or what you highlighted more than 10 seconds ago.
- **Queue**: type prompts and copy them with Enter. Links open with a click, and every item has a "Copy" button.
- **Notes**: a free-form scratchpad that saves itself.
- **History**: items you clear or delete from the queue are kept, searchable, and can be restored.
- **With the browser extension** (Brave, Chrome or Chromium):
  - Captures remember their page: favicon, domain, title, and a link back to it.
  - With no fresh selection, the active tab's URL is captured, which also covers the address bar.
  - On WhatsApp Web it saves the sender, the group and when the message was sent.
  - Text is saved as it looks on screen. Chromium breaks lines around inline-flex elements such as X mentions; Patina fixes that.

## Compatibility

| | Status |
| --- | --- |
| Hyprland 0.55+ with `hyprland.lua` | Tested on 0.56 |
| Hyprland 0.53+ with `hyprland.conf` | Config verified with `Hyprland --verify-config` on 0.56 |
| Hyprland 0.48–0.52 | Config written from the official docs for those versions, untested |
| Hyprland 0.47 and older | Config written from the official docs (`windowrulev2`), untested |
| Other Wayland compositors (Sway, niri, KDE, GNOME) | Not supported: Patina uses `hyprctl` to read the active window |
| macOS | Not supported. Use [Copper](https://shadcn.com/copper) |

## Requirements

- Hyprland
- Python 3 with GTK 4 and libadwaita
- `wl-clipboard` and `libnotify`

On Fedora:

```sh
sudo dnf install python3-gobject gtk4 libadwaita wl-clipboard libnotify
```

On Arch:

```sh
sudo pacman -S python-gobject gtk4 libadwaita wl-clipboard libnotify
```

## Install

```sh
git clone https://github.com/lautarosegura/patina
cd patina
./install.sh
```

`install.sh` links `~/.local/bin/patina` to the file in your clone and registers the native messaging host for every Chromium-based browser it finds. The extension and the Hyprland config are used straight from the clone, so keep it around; if you move it, run `install.sh` again and update the paths. It doesn't use `sudo` and doesn't edit your config.

### Hyprland

`install.sh` detects your Hyprland version and config format and prints the exact line to add at the end of your config. Then reload with `hyprctl reload`.

| Your setup | Line to add |
| --- | --- |
| 0.55+ with `hyprland.lua` | `dofile("/path/to/patina/hypr/patina.lua")` |
| 0.53+ with `hyprland.conf` | `source = /path/to/patina/hypr/patina.conf` |
| 0.48–0.52 | `source = /path/to/patina/hypr/patina-0.48.conf` |
| 0.47 and older | `source = /path/to/patina/hypr/patina-0.47.conf` |

This adds double Shift capture, **Super+C** to show or hide Patina, a floating window rule, and autostart on login.

If you installed the command somewhere else with `BIN_DIR=... ./install.sh`, set `PATINA_BIN = "/path/to/patina"` before the `dofile`. With a `.conf` file, edit the `~/.local/bin/patina` paths in it instead.

### Browser extension (optional)

1. Open `brave://extensions` or `chrome://extensions`.
2. Turn on **Developer mode**.
3. Click **Load unpacked** and pick the `extension/` folder of your clone.
4. Reload the tabs you already had open.

The extension asks for:

- **Read site data**: to know which page the highlighted text comes from.
- **Tabs**: shown by the browser as "read your browsing history". It's used to get the active tab's URL and title.
- **Favicons**: to read the icons the browser already has cached, without fetching them.
- **Native messaging**: to hand the data to Patina.

## Usage

| Action | Shortcut |
| --- | --- |
| Capture the highlighted text | Shift twice (within 350 ms) |
| Show or hide Patina | Super+C |
| Add a prompt | Type in the field and press Enter |
| Move from the field to the list | ↓ or Tab |
| Copy and go back to the previous app | Enter, Ctrl+C or double-click on an item |
| Copy only | "Copy" button shown on hover |
| Check or uncheck | Checkbox or Space |
| Open a link or the source page | Click the link or the domain |
| Delete (moves to history) | Delete or Backspace |
| Reorder | Ctrl+↑ / Ctrl+↓ |
| Clear checked items (moves them to history) | Ctrl+Shift+Backspace |
| Queue / Notes / History | Ctrl+1 / Ctrl+2 / Ctrl+3 |
| History: restore to the queue | Ctrl+Z or the ↶ button |
| History: delete forever | Delete |
| Hide | Esc |

From a terminal, `patina add "text"` adds a prompt and `patina capture` captures the current selection.

## Data

- `~/.local/share/patina/`: `items.json` (the queue), `historial.json` (history), `notas.md` (notes) and `favicons/`.
- `$XDG_RUNTIME_DIR/patina-*.json`: the latest selection and active tab sent by the extension. These live in memory, are `0600`, and are gone when you log out.

## How it works

- **One program:** `patina` is a single Python file using GTK 4 and libadwaita. It stays running hidden (`patina --service`), and other commands reach it over D-Bus.
- **Double Shift:** Hyprland runs `patina tap` when Shift is released, without consuming the key. `tap` doesn't load GTK; two taps within 350 ms run `patina capture`.
- **Stale selections:** on Wayland the primary selection stays available even after nothing looks selected. Patina watches it with `wl-paste --primary --watch` to ignore stale selections.
- **Extension:** `content.js` sends the highlighted text and, on WhatsApp Web, the sender (WhatsApp tags messages with `data-pre-plain-text`). `background.js` adds the favicon and the active tab and forwards everything over native messaging to the `io.github.lautarosegura.patina` host.
- **Native host:** the same `patina` file, launched by the browser with a `chrome-extension://…` argument. It writes to `$XDG_RUNTIME_DIR`, and a capture only uses that data when it matches the captured text.
- **Extension ID:** `manifest.json` includes a public key, so the extension ID (`fedemognlmdojgmeinfhhjliccpfncdo`) is fixed and matches the one the native host allows.

## Development

The command, the extension and the Hyprland config all point at the clone, so your changes are live:

- **`patina`:** restart the service with `pkill -f "patina --service"; patina --service &`.
- **`extension/`:** reload it in `brave://extensions`.
- **`hypr/`:** run `hyprctl reload`.

## Uninstall

```sh
./uninstall.sh          # removes the command and the native host, keeps your data
./uninstall.sh --purge  # also deletes ~/.local/share/patina
```

## License

[MIT](LICENSE). The WhatsApp logo comes from [Simple Icons](https://simpleicons.org) (CC0).
