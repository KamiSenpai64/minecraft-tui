# Minecraft TUI

A terminal user interface (TUI) for managing and launching Minecraft instances via PrismLauncher.

## Features

### Core Functionality
- ✅ View all PrismLauncher instances
- ✅ Display playtime and last played information
- ✅ Launch instances directly from the TUI
- ✅ Keyboard navigation (↑↓ or j/k)

### New Features (In Development)

- [x] **#1: Open instance folder** - Press 'o' to open the instance directory in your file manager
- [x] **#2: Show Minecraft version** - Display which MC version each instance uses
- [x] **#3: Sort options** - Press 's' to cycle through sorting: by name, last played, or playtime
- [x] **#4: Search/Filter** - Press '/' to filter instances by name as you type
- [x] **#9: Instance details view** - Press 'i' to see full details (version, loader, mods, etc.)
- [x] **#12: Show if instance is running** - Visual indicator if Minecraft is already running

## Requirements

- PrismLauncher (installed via Flatpak)
- Rust toolchain (for building)

## Installation

```bash
# Clone the repository
git clone git@github.com:KamiSenpai64/minecraft-tui.git
cd minecraft-tui

# Build the project
cargo build --release

# Create the launch script directory
mkdir -p ~/scripts

# Make sure the launch script is executable
chmod +x ~/scripts/launch-minecraft.sh

# Run the application
./target/release/minecraft-tui
```

## Usage

- **↑/↓** or **j/k** - Navigate through instances
- **Enter** - Launch selected instance
- **q** or **Esc** - Quit

## Project Structure

- `src/main.rs` - Main TUI application
- `~/scripts/launch-minecraft.sh` - Script for launching Minecraft instances

## License

MIT
