#!/bin/bash
# Launch Minecraft instance via PrismLauncher
# Usage: launch-minecraft.sh "instance-name"

if [ -z "$1" ]; then
    echo "Usage: $0 <instance-name>"
    exit 1
fi

# Launch the instance completely detached using nohup
nohup flatpak run org.prismlauncher.PrismLauncher --launch "$1" >/dev/null 2>&1 &
disown
