# Systema - Windows PC Optimizer

A lightweight, modern Windows PC optimization tool built with Tauri (Rust + React). Systema provides a user-friendly interface to optimize your Windows PC with just a few clicks.

## Features

### System Optimization
- **Virtual Memory (RAM Swap)**: Configure your SSD as backup RAM to prevent crashes during heavy usage
- **Windows Services**: Disable or set to manual unnecessary Windows services
- **Startup Apps**: Control which applications launch at Windows startup
- **Power Settings**: Switch to high-performance power mode

### Privacy & Security
- **Telemetry Settings**: Disable Windows advertising ID, app tracking, and suggested content
- **Windows Defender**: Enable Controlled Folder Access for ransomware protection
- **DNS Settings**: Configure Cloudflare DNS (1.1.1.1) for faster and more private browsing

### Performance Tweaks
- **Optional Features**: Remove unused Windows features (IE11, Fax, legacy components)
- **Visual Effects**: Reduce Windows animations while keeping essential elements
- **Quick Optimize**: One-click optimization applying safe, recommended settings

### Tool Guides
- Step-by-step guides for Process Lasso, Revo Uninstaller, and Edge removal

## Building

### Prerequisites

1. **Rust**: Install from [rustup.rs](https://rustup.rs/)
2. **Node.js**: Version 18+ recommended
3. **Windows SDK**: For Windows build targets

### Development

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Production Build

```bash
# Build for Windows (creates .msi and .exe installers)
npm run tauri build
```

The built installers will be in `src-tauri/target/release/bundle/`.

## Tech Stack

- **Backend**: Rust with Tauri 2.0
- **Frontend**: React 19 with TypeScript
- **Build Tool**: Vite
- **Styling**: Custom CSS with modern dark theme

## Safety Notes

- All optimizations are safe and reversible
- The app requires administrator privileges for some operations
- Always read the descriptions before disabling services
- Changes to virtual memory and optional features may require a restart

## License

MIT License
