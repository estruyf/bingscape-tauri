# Bingscape

A lightweight Tauri desktop app that automatically fetches and sets the daily
Bing image as your macOS wallpaper.

## Features

- **Automatic Updates**: Syncs the latest Bing daily image every hour
- **Manual Sync**: Trigger wallpaper updates anytime with a single click
- **Multi-Display Support**: Apply wallpaper to all displays or just the main
  one
- **Background Operation**: Runs without a dock icon; access via system tray
- **Start at Login**: Optional auto-start when you log in
- **Settings Persistence**: Your preferences are saved across restarts

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

[Your License Here]

