# Minecraft TUI

![GIF of the application](https://media2.giphy.com/media/v1.Y2lkPTc5MGI3NjExNThrYWwxazBrNWh5d3E2OTVmejVrdnB1cjdsMHZxbjhld2pjNXRyYSZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/8UY5okxFuO0LO0Upxr/giphy.gif)

A terminal-based user interface for managing and launching Minecraft instances through PrismLauncher. Built with Rust and Ratatui for a fast, keyboard-driven experience.

## Prerequisites

- **PrismLauncher** - Installed via Flatpak
- **Rust toolchain** - Version 1.70 or higher (for building from source)
- **Linux** - Tested on Ubuntu/Debian-based systems
- **xdg-open** - For opening instance folders

## Installation

### Quick Install (Recommended)

```bash
# Clone the repository
git clone git@github.com:KamiSenpai64/minecraft-tui.git
cd minecraft-tui

# Run the install script
./install.sh
```

The install script will:
- Build the release binary
- Copy the launch script to `~/scripts`
- Install the binary to `~/.local/bin`
- Check for required dependencies

### Manual Installation

```bash
# Clone the repository
git clone git@github.com:KamiSenpai64/minecraft-tui.git
cd minecraft-tui

# Build the release binary
cargo build --release

# Copy the launch script
mkdir -p ~/scripts
cp scripts/launch-minecraft.sh ~/scripts/
chmod +x ~/scripts/launch-minecraft.sh

# Run the application
./target/release/minecraft-tui
```

### Optional: Add to PATH

```bash
# Copy binary to local bin
mkdir -p ~/.local/bin
cp target/release/minecraft-tui ~/.local/bin/

# Add to PATH (add to ~/.bashrc or ~/.zshrc)
export PATH="$HOME/.local/bin:$PATH"

# Now you can run from anywhere
minecraft-tui
```

## How to Use

### Basic Navigation

- `↑` / `↓` or `j` / `k` - Navigate through instances
- `Enter` - Launch the selected instance
- `q` or `Esc` - Quit the application

### Features

- `o` - Open instance folder in file manager
- `s` - Cycle sort mode (Name → Last Played → Playtime)
- `/` - Enter search mode to filter instances
- `i` - Toggle instance details panel
- `Backspace` - Delete search query character (in search mode)
- `Esc` - Exit search mode (when searching)

## Preview

Screenshots and demo GIFs coming soon! Run `minecraft-tui` to see it in action.

## Implemented Features

1. [x] **View All Instances** - Display all PrismLauncher instances with metadata
2. [x] **Launch Instances** - Start Minecraft instances directly from the TUI
3. [x] **Keyboard Navigation** - Navigate with arrow keys or vim-style (j/k)
4. [x] **Display Metadata** - Show playtime and last played information
5. [x] **Minecraft Version Display** - Show MC version next to instance name
6. [x] **Mod Loader Detection** - Detect and display mod loader (Fabric, Forge, Quilt, NeoForge, Vanilla)
7. [x] **Open Instance Folder** - Quick access to instance directory
8. [x] **Multiple Sort Modes** - Sort by name, last played, or total playtime
9. [x] **Real-time Search/Filter** - Filter instances as you type
10. [x] **Instance Details Panel** - View comprehensive details in split view with mod count
11. [x] **Running Status Indicator** - Visual indicator for active instances
12. [x] **Automated CI/CD** - GitHub Actions for building and testing
13. [x] **Unit Tests** - Test coverage for core functionality
14. [x] **Easy Installation** - One-command install script

## Upcoming Features

15. [ ] **Refresh Instances** - Reload instance list without restarting (press 'r')
16. [ ] **Favorites System** - Pin favorite instances to the top
17. [ ] **Multiple Instance Launch** - Select and launch multiple instances
18. [ ] **Configuration File** - Save user preferences and settings
19. [ ] **Theme Support** - Customizable color schemes
20. [ ] **Launch History** - Track and display launch history
21. [ ] **Java Version Display** - Show which Java version each instance uses
22. [ ] **World Count** - Display number of worlds per instance
23. [ ] **Multi-Launcher Support** - Support for MultiMC, ATLauncher, etc.

## Project Structure

```
minecraft-tui/
├── src/
│   └── main.rs          # Main application code
├── Cargo.toml           # Rust dependencies
├── README.md            # This file
└── target/
    └── release/
        └── minecraft-tui # Compiled binary
```

### Launch Script

The application uses a bash script (`~/scripts/launch-minecraft.sh`) to launch instances in a detached process, ensuring Minecraft continues running after the TUI exits.

## Troubleshooting

### Instance Not Launching

If pressing Enter doesn't launch an instance:
- Verify PrismLauncher is installed: `flatpak list | grep Prism`
- Check the launch script exists: `ls ~/scripts/launch-minecraft.sh`
- Ensure the script is executable: `chmod +x ~/scripts/launch-minecraft.sh`

### Instances Not Showing

If no instances appear:
- Verify PrismLauncher path: `~/.var/app/org.prismlauncher.PrismLauncher/data/PrismLauncher/instances`
- Ensure you have at least one instance created in PrismLauncher

### Running Indicator Not Working

The running status check uses `ps aux` to detect active Minecraft processes. If it's not working:
- Verify processes are visible: `ps aux | grep flatpak | grep PrismLauncher`

### Releases
The reccomended way of using minecraft-tui is downloading the latest release, as it does not have the README.md file, LICENSE file, etc.

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

MIT
