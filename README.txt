CAPTURE CARD VIEWER
by Quidque Studio
===============================================

FIRST LAUNCH
  Select your Video Input and Audio Input from
  the dropdowns, then click Save to start.

  Video Input  - usually "USB Video", "USB Capture",
                 or your capture card's brand name
  Audio Input  - usually matches the video device name
                 or uses same the guidelines

CONTROLS
  F11   Toggle fullscreen
  S     Open / close settings

SETTINGS (press S)
  Video Input   Switch video source
  Audio Input   Switch audio source
  Audio Output  Where to send audio (default: system)
  Volume        0x to 3x; values above 1.0 amplify

TROUBLESHOOTING
  No devices shown
    Make sure the capture card is plugged in before
    launching. Check Device Manager for driver issues.

  Disconnects immediately after loading
    Another app (e.g. OBS Studio) is using the device.
    Close it, then restart "Capture Card Viewer".

  No audio
    Check that the right Audio Output is selected in
    Settings and that volume is above zero.

  Black screen or frozen frame
    Ensure the source device is powered on and actively
    outputting video to the capture card.

DATA & LOGS
  All files are stored at:
  %LOCALAPPDATA%\capture-card-viewer\

    settings.toml   Saved device and display settings
    app.log         Log from the most recent session

  Settings are saved automatically when the app closes.
  A corrupt settings file is reset automatically.

===============================================
© 2026 Quidque Studio
