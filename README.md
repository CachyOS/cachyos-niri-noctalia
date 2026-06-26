# niri-clipboard

Clipboard history manager for Niri/Wayland with themed wofi picker.

## Features

- **Super+V** opens clipboard history picker
- Content type indicators: `[IMG 1920x1080]`, `[VIDEO]`, `[AUDIO]`, `[PDF]`, `[FILE]`
- Text preview for copied text entries
- File metadata display (name, size, MIME type)
- **Occult Umbral** themed wofi styling matching noctalia-shell
- Auto-paste via simulated Ctrl+Shift+V

## Dependencies

- `cliphist` - Wayland clipboard history storage
- `wofi` - Application launcher/picker
- `imagemagick` - Image dimension detection
- `file` - MIME type detection
- `wtype` - Wayland keyboard input simulation
- `coreutils` - Basic utilities (du, basename)

## Usage

```bash
# Start the clipboard store daemon (usually auto-started)
niri-clipboard store

# Open clipboard history picker
niri-clipboard pick

# List clipboard history
niri-clipboard list

# Wipe clipboard history
niri-clipboard wipe
```

## Keybind

Add to your `keybinds.kdl`:

```
Mod+V { spawn "niri-clipboard" "pick"; }
```

## Autostart

Add to your `autostart.kdl`:

```
spawn-sh-at-startup "niri-clipboard store"
```
