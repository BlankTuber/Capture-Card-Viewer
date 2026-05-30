# Capture Card Viewer

A lightweight capture card viewer for Windows. Displays video and audio from standard USB capture cards with minimal setup and no bloat.

---

## Requirements

- Windows 10 or later (64-bit)
- A USB capture card with UVC (USB Video Class) support
- Any drivers your capture card's manufacturer recommends installed first

---

## Installation

Run `CaptureCardViewer-Setup.exe` and follow the on-screen prompts. A shortcut will be added to your Start Menu.

---

## First Launch

On first launch you will be asked to select your devices:

- **Video Input** — usually listed as "USB Video", "USB Capture", or your capture card's brand name (e.g. "Elgato HD60 S", "AVerMedia Live Gamer")
- **Audio Input** — usually shares a name with the video input device, or follows the same guidelines

Click **Save** to confirm your selections and start the viewer.

---

## Controls

|        Action         | Keybind |
|-----------------------|---------|
|   Toggle fullscreen   | **F11** |
| Open / close settings |  **S**  |

The window can also be closed using the **✖** button in the top-right corner (visible when not in fullscreen).

---

## Settings

Press **S** or click **Settings** in the menu bar to open the settings panel.

|     Setting      |                                     Description                                     |
|------------------|-------------------------------------------------------------------------------------|
| **Video Input**  |                          Your capture card's video source                           |
| **Audio Input**  |                          Your capture card's audio source                           |
| **Audio Output** |           Where to send the audio — defaults to your system default output          |
|    **Volume**    | Playback volume (0× – 3×). Values above 1.0 amplify the signal; 1.5× is the default |

Click **Save** to apply your changes. If a device was changed, the stream will restart automatically.
Click **Close** to discard any unsaved changes and return to the viewer.

---

## Troubleshooting

### No devices appear in the dropdown

Make sure the capture card is connected and recognized by Windows *before* launching the app. Open Device Manager and check that it appears without errors. Try a different USB port if needed.

### Stream disconnects immediately after loading

Another application — such as OBS Studio, Streamlabs, or a browser — is most likely already using the capture card. Close those applications first, then go to **Settings → Save** to reconnect.

### No audio coming through

- Confirm the correct **Audio Output** and **Audio Input** is selected in Settings.
- Make sure the volume slider is above zero.
- Check that the connected source device (console, PC, etc.) is actually outputting audio to the capture card.

### Black screen or frozen frame

Some capture cards require an active incoming signal before they expose a stream. Make sure the source device is powered on and actively outputting video.

### App won't open or crashes on startup

The log file contains diagnostic information that may help identify the cause. You can find it at:

```console
%LOCALAPPDATA%\capture-card-viewer\app.log
```

---

## Data & Logs

All app data is stored locally at:

```console
%LOCALAPPDATA%\capture-card-viewer\
```

|      File       |                          Description                          |
|-----------------|---------------------------------------------------------------|
| `settings.toml` | Your saved device selections, volume, and display preferences |
|    `app.log`    |            Log output from the most recent session            |

Settings are saved automatically when the app closes.
If `settings.toml` becomes corrupted for any reason, it will be reset automatically on next launch and you will be taken back to the first-launch setup screen.

---

*Capture Card Viewer — © 2026 Quidque Studio*
