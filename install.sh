#!/bin/bash
# Minecraft TUI Installation Script

set -e

echo "=== Minecraft TUI Installer ==="
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Cargo not found. Please install Rust first:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check if PrismLauncher is installed
if ! flatpak list | grep -q "org.prismlauncher.PrismLauncher"; then
    echo "Warning: PrismLauncher not detected via Flatpak"
    echo "  Install with: flatpak install flathub org.prismlauncher.PrismLauncher"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo "Building minecraft-tui..."
cargo build --release

echo ""
echo "Installing launch script..."
mkdir -p ~/scripts
cp scripts/launch-minecraft.sh ~/scripts/
chmod +x ~/scripts/launch-minecraft.sh

echo ""
echo "Installing binary..."
mkdir -p ~/.local/bin
cp target/release/minecraft-tui ~/.local/bin/

# Check if ~/.local/bin is in PATH
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    echo ""
    echo "Warning: ~/.local/bin is not in your PATH"
    echo "Add this to your ~/.bashrc or ~/.zshrc:"
    echo '  export PATH="$HOME/.local/bin:$PATH"'
    echo ""
    echo "Then run: source ~/.bashrc (or ~/.zshrc)"
fi

echo ""
echo "=== Installation Complete! ==="
echo ""
echo "Run 'minecraft-tui' to start"
echo ""
