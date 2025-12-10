<div align="center">
  <img src="app-icon.png" alt="Bingscape Icon" width="128" height="128">
                                                
  # Bingscape
                                                
  **A lightweight Tauri desktop app that automatically fetches and sets the
  daily Bing image as your macOS wallpaper.**
    
  
  [![macOS](https://img.shields.io/badge/macOS-10.15+-blue.svg)](https://www.apple.com/macos/)
  [![Tauri](https://img.shields.io/badge/Tauri-2.0-24C8DB.svg)](https://tauri.app/)
  [![TypeScript](https://img.shields.io/badge/TypeScript-5.0-3178C6.svg)](https://www.typescriptlang.org/)
  [![React](https://img.shields.io/badge/React-19.1-61DAFB.svg)](https://reactjs.org/)
                                                
  <img src="https://img.shields.io/badge/License-MIT-green.svg" alt="License">
</div>

## Features

- **Automatic Updates**: Syncs the latest Bing daily image every hour
- **Manual Sync**: Trigger wallpaper updates anytime with a single click
- **Multi-Display Support**: Apply wallpaper to all displays or just the main
  one
- **Background Operation**: Runs without a dock icon; access via system tray
- **Start at Login**: Optional auto-start when you log in
- **Settings Persistence**: Your preferences are saved across restarts

## 📸 Screenshots

<div align="center">
  <img src="screenshot.png" alt="Bingscape Main Interface" width="600">
  <p><em>Clean, intuitive interface to manually or automatically update your
  wallpaper.</em></p>
</div>

## Getting Started

### Prerequisites

- Node.js 20.19+ or 22.12+
- Rust (latest stable)
- macOS (this app uses macOS-specific APIs)

### Installation

```bash
# Clone the repository
git clone <your-repo-url>
cd Bingscape

# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## Usage

1. **Launch the app** - On first run, it fetches and applies the current Bing
   wallpaper
2. **System Tray** - Click the tray icon to show/hide the app window;
   right-click for menu
3. **Settings**:
   - Toggle **Hourly polling** to enable/disable automatic updates
   - Toggle **Apply to all displays** to set wallpaper on all screens
   - Toggle **Start at login** to launch automatically when you log in
4. **Manual Sync** - Click "Sync now" to immediately fetch the latest wallpaper
5. **View Status** - Check last run time, target displays, and open the image or
   saved file location

## Tech Stack

- **Frontend**: React + TypeScript + Vite
- **Backend**: Rust + Tauri v2
- **Plugins**:
  - `tauri-plugin-store` for settings persistence
  - `tauri-plugin-autostart` for login startup
  - `tauri-plugin-shell` for opening files/URLs

## Development

```bash
# Run dev server
npm run tauri dev

# Build for production
npm run tauri build

# Lint/format (if configured)
npm run lint
```

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) +
  [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
  +
  [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## License

MIT License

Copyright (c) 2025 Elio Struyf

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.


<div align="center">
  Made with ❤️ for macOS developers and designers
                          
  **[⬆ back to top](#bingscape)**
</div>

<div align="center">
   <a
   href="https://visitorbadge.io/status?path=https%3A%2F%2Fgithub.com%2Festruyf%2Fbingscape"><img
   src="https://api.visitorbadge.io/api/visitors?path=https%3A%2F%2Fgithub.com%2Festruyf%2Fbingscape&countColor=%23263759"
   /></a>
</div>