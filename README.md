# NotchIsland 🏝️

**NotchIsland** is an authentic, high-performance Windows Dynamic Island utility built natively in **C# / .NET 9 and WPF**, reverse-engineered directly from **NotchPixel** (`KOREINDUSTRIES.NotchPixel`).

It delivers deep OLED styling, continuous squircle curvature, live media playback integration, a full 10-tab navigation system, and a comprehensive native Settings window.

---

## ✨ Features

- **🖤 Authentic OLED Black Island**:
  - Continuous squircle curvature with seamless desktop transparency (`DwmExtendFrameIntoClientArea` + `WS_EX_TOOLWINDOW | WS_EX_TOPMOST`).
  - **Compact Pill Mode** (320x40): Live mini album art or audio visualizer bars, animated desktop pet (`(=^･ω･^=)`), dynamic title/artist & clock, live battery % and status.
  - **Expanded Card Mode** (560x270): 10 fully functional tabs and custom settings.

- **🎵 1. Music (Media)**:
  - Windows Global System Media Transport Controls (GSMTC) integration.
  - Real album art thumbnail, title, artist, and album.
  - Interactive seek slider with live elapsed and remaining times.
  - Play/Pause, Skip Next, Skip Previous, and Master Volume controls.

- **📁 2. File Shelf**:
  - Live drag-and-drop zone from Windows File Explorer.
  - File management: quick preview, file size formatting, "Open" in default app, and "Remove".
  - Persistent shelf file holding.

- **📋 3. Clipboard Manager**:
  - Real-time Win32 clipboard monitoring.
  - Text snippet history with timestamps.
  - Pinning, single-click copy back to clipboard, individual deletion, and history clearing.

- **🧩 4. Widgets**:
  - **Digital Clock**: Live HH:mm:ss display with localized date.
  - **Live Weather**: Temperature, weather condition, and air quality report.
  - **Hardware Monitor**: Real CPU % (`PerformanceCounter`), RAM usage and total GB (`GlobalMemoryStatusEx`), battery level, and AC power state.
  - **Stopwatch / Timer**: Interactive Start, Pause, and Reset controls with millisecond precision.

- **🎛 5. Controls & System**:
  - Real Master Volume slider with live Mute/Unmute toggle (via CoreAudio COM `IAudioEndpointVolume`).
  - Monitor Brightness slider (via WMI `WmiMonitorBrightnessMethods`).
  - Quick-launch shortcuts for Windows Wi-Fi, Bluetooth, and Night Light settings.

- **🌐 6. Web Quick Launch**:
  - Customizable URL launcher bar with automatic browser invocation.
  - Quick launch tiles: Google, YouTube, GitHub, ChatGPT, and Reddit.

- **🔔 7. Notifications**:
  - Windows Notification Mirroring feed with source header, message body, and dismiss/clear all.

- **✂ 8. Tools & Screen Capture**:
  - Full-screen screenshot tool: captures entire screen, saves PNG to `Pictures\Screenshots`, copies image to clipboard, and automatically places it on the File Shelf.
  - Screen Snip tool: launches Windows Snip & Sketch (`ms-screenclip:`).
  - Screen Color Picker: magnifies pixel under cursor and copies Hex `#RRGGBB` to clipboard.
  - Task Manager quick launcher.

- **📱 9. PC Deck Companion**:
  - Embedded local HTTP server on `http://localhost:8088`.
  - Serves authentic extracted Deck companion web interface (`app.js`, `style.css`, `pad.js`, `pages.js`).
  - Live QR code generated on-screen for instant mobile device pairing over Wi-Fi.

- **🎧 10. Calls & Audio**:
  - Interactive Microphone Mute / Live toggle with instant status indicator.
  - Endpoint monitoring.

- **⚙ Comprehensive Settings Window**:
  - Dedicated 840x680 OLED dark settings window matching NotchPixel's exact architecture.
  - 8 configuration categories: General, Appearance, Media & Music, Shelf & Clipboard, Widgets, Controls & HUDs, Hotkeys, PC Deck, and About.
  - Automatically loads and persists to `%APPDATA%\NotchIsland\settings.json`.

---

## 🚀 Download & Run

Download the pre-compiled, self-contained single-file executable from [Releases](https://github.com/abhinavstechdev/NotchIsland/releases):
- **`NotchIsland.exe`** (~197 MB, self-contained, no .NET runtime installation required).

Run `NotchIsland.exe` directly on Windows 10 or Windows 11 (x64).
