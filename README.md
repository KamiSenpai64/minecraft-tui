# Minecraft TUI

A terminal user interface (TUI) for managing and launching Minecraft instances via PrismLauncher.

## Features

### Core Functionality
- ✅ View all PrismLauncher instances
- ✅ Display playtime and last played information
- ✅ Launch instances directly from the TUI
- ✅ Keyboard navigation (↑↓ or j/k)

### New Features (In Development)

- [ ] **#1: Open instance folder** - Press 'o' to open the instance directory in your file manager
- [ ] **#2: Show Minecraft version** - Display which MC version each instance uses
- [ ] **#3: Sort options** - Press 's' to cycle through sorting: by name, last played, or playtime
- [ ] **#4: Search/Filter** - Press '/' to filter instances by name as you type
- [ ] **#5: Refresh list** - Press 'r' to reload instances without restarting the app
- [ ] **#6: Show mod count** - Display how many mods each instance has
- [ ] **#7: Favorites system** - Press 'f' to favorite instances, show them at the top
- [ ] **#9: Instance details view** - Press 'i' to see full details (version, loader, mods, etc.)
- [ ] **#11: Multiple instance launch** - Mark instances with spacebar, launch them all
- [ ] **#12: Show if instance is running** - Visual indicator if Minecraft is already running

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
