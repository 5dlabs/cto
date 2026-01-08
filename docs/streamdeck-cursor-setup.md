# Stream Deck Setup for Cursor IDE

This guide covers setting up Stream Deck (including Mobile) buttons for quick access to Cursor IDE features.

## Cursor Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd + I` | Open **Composer** (main AI agent) - cursor in prompt |
| `Cmd + L` | Open **Chat** sidebar - cursor in prompt |
| `Cmd + K` | Open **Inline Edit** at cursor position |
| `Cmd + Shift + I` | Toggle Composer panel |

## Adding a "Cursor Agent" Button

### Option 1: Via Stream Deck UI (Recommended)

1. Open **Stream Deck** app
2. Drag **"Hotkey"** action (under System) to an empty slot
3. Click the button to configure:
   - **Title**: `Agent` (or `Composer`)
   - **Hotkey**: Press `Cmd + I` in the recording field
4. Optionally set an icon

### Option 2: Multi-Action (Open Cursor + Agent)

Create a sequence that opens Cursor and immediately triggers the agent:

1. Drag **"Multi Action"** to a slot
2. Add these actions in order:
   - **Open Application**: Cursor
   - **Delay**: 500ms (wait for app to focus)
   - **Hotkey**: `Cmd + I`
3. Title it `Cursor Agent`

### Option 3: Manual Profile Edit

Stream Deck profiles are stored at:
```
~/Library/Application Support/com.elgato.StreamDeck/ProfilesV3/
```

To add a Cmd+I hotkey action, add this to the `Actions` object in the profile's `manifest.json`:

```json
"0,1": {
  "ActionID": "cursor-agent-action-id",
  "LinkedTitle": true,
  "Name": "Hotkey",
  "Plugin": {
    "Name": "Activate a Key Command",
    "UUID": "com.elgato.streamdeck.system.hotkey",
    "Version": "1.0"
  },
  "Settings": {
    "Coalesce": true,
    "Hotkeys": [
      {
        "KeyCmd": true,
        "KeyCtrl": false,
        "KeyModifiers": 8,
        "KeyOption": false,
        "KeyShift": false,
        "NativeCode": 34,
        "QTKeyCode": 73,
        "VKeyCode": 34
      },
      {"KeyCmd": false, "KeyCtrl": false, "KeyModifiers": 0, "KeyOption": false, "KeyShift": false, "NativeCode": -1, "QTKeyCode": 33554431, "VKeyCode": -1},
      {"KeyCmd": false, "KeyCtrl": false, "KeyModifiers": 0, "KeyOption": false, "KeyShift": false, "NativeCode": -1, "QTKeyCode": 33554431, "VKeyCode": -1},
      {"KeyCmd": false, "KeyCtrl": false, "KeyModifiers": 0, "KeyOption": false, "KeyShift": false, "NativeCode": -1, "QTKeyCode": 33554431, "VKeyCode": -1}
    ]
  },
  "State": 0,
  "States": [{"Title": "Agent"}],
  "UUID": "com.elgato.streamdeck.system.hotkey"
}
```

After editing, restart Stream Deck:
```bash
killall "Stream Deck" && open -a "Stream Deck"
```

## Key Codes Reference (macOS)

| Key | NativeCode | QTKeyCode |
|-----|------------|-----------|
| I | 34 | 73 |
| L | 37 | 76 |
| K | 40 | 75 |
| V | 9 | 86 |
| A | 0 | 65 |

| Modifier | KeyModifiers value |
|----------|-------------------|
| Cmd | 8 |
| Shift | 2 |
| Option | 4 |
| Ctrl | 1 |

## Current Profile Location

Your Stream Deck Mobile profile:
```
~/Library/Application Support/com.elgato.StreamDeck/ProfilesV3/FF506043-179A-49ED-A036-6BC8386AE05E.sdProfile/
```
