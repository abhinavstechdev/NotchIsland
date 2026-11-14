# NotchIsland 🏝️ — Native Windows Dynamic Island & Notch

A blazing-fast, lightweight native desktop customization app written in **Rust** that brings an interactive Dynamic Island and top notch to **Windows 10 & 11**.

Inspired by Apple's Dynamic Island and reverse-engineered from NotchPixel using REA, **NotchIsland** is built from scratch with pure Win32 windowing, analytical spring physics, fluid continuous-curvature squircles, local Wi-Fi file sharing, and hardware telemetry.

---

## ✨ Features

- **🏝️ Dynamic Island & Bezel Notch**: Sits discreetly at the top-center of your display. Expands fluidly upon cursor hover.
- **🖱️ True Click-Through & Zero Focus Stealing**:
  - Uses `WS_EX_TRANSPARENT` in idle mode so clicks pass directly through to windows beneath.
  - Active proximity detection removes click-through when hovered.
  - Configured with `WS_EX_NOACTIVATE` so it never steals keyboard focus from full-screen games, code editors, or typing.
- **🎵 Interactive Media Player**:
  - Windows Global System Media Transport Controls (GSMTC) integration.
  - Track metadata, dynamic album accent tinting, seek scrubber slider, and animated frequency bars in pill mode.
- **📁 File Shelf & Wireless Phone Transfer**:
  - Drag and drop or stage files directly onto the island.
  - Embedded background HTTP server on local Wi-Fi.
  - Generates a 2D QR Code directly on the card: scan with your phone's camera to download files without an app or internet connection!
- **⚡ Performance Telemetry & Gaming HUD**:
  - Real-time Hardware FPS, CPU load, and RAM usage gauges.
  - Automatically collapses into an ultra-slim 28px gaming HUD when fullscreen DirectX/D3D applications or games are active.
- **🎛️ Quick System Utilities**:
  - Master volume slider & mute toggle.
  - Global microphone mute button (`Ctrl + Alt + M`).
  - Screen backlight brightness control (VESA DDC/CI).
  - One-click switcher between Floating Island and Top Bezel Notch.

---

## 📐 Mathematical & Architectural Highlights

- **Squircle Geometry**: Renders G2 continuous curvature superellipses using cubic Bézier offsets with $\kappa = \frac{4(\sqrt{2}-1)}{3} \approx 0.55228$.
- **Analytical Spring Physics**:
  $$x(t) = 1.0 - e^{-(t / 16.0)^2} \cdot \left[ 1.4 \cdot A \cdot \sin(4.0 \cdot t - 0.5) + 0.18 \cdot A \cdot \sin(7.5 \cdot t) \right]$$
- **High Performance**: Native binary consuming ~14 MB RAM with sub-millisecond frame rendering.

---

## 🚀 Building & Running

### Prerequisites
- [Rust & Cargo](https://rustup.rs/) (1.80+)
- Windows 10 (version 19041+) or Windows 11 (x64)

### Development
```bash
git clone https://github.com/abhinavstechdev/NotchIsland.git
cd NotchIsland
cargo run
```

### Build Optimized Release Executable
```bash
cargo build --release
```
The standalone `.exe` will be located at `target/release/notch_island.exe` with no terminal console window.

---

## 📜 License
MIT License. Free and open source.
