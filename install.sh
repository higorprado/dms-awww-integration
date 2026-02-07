#!/bin/bash
set -e

INSTALL_DIR="$HOME/.cargo/bin"
SERVICE_FILE="systemd/dms-awww.service"
USER_SYSTEMD_DIR="$HOME/.config/systemd/user"
ENV_DIR="$HOME/.config/dms-awww"

echo "Building dms-awww..."
cargo build --release

echo "Installing binary to $INSTALL_DIR..."
install -Dm755 target/release/dms-awww "$INSTALL_DIR/dms-awww"

echo "Installing systemd service..."
mkdir -p "$USER_SYSTEMD_DIR"
cp "$SERVICE_FILE" "$USER_SYSTEMD_DIR/"

echo "Setting up Wayland environment..."
mkdir -p "$ENV_DIR"
cat > "$ENV_DIR/environment" << EOF
WAYLAND_DISPLAY=$WAYLAND_DISPLAY
DBUS_SESSION_BUS_ADDRESS=$DBUS_SESSION_BUS_ADDRESS
EOF

echo "Reloading systemd daemon..."
systemctl --user daemon-reload

echo ""
echo "Installation complete!"
echo ""
echo "To enable and start the service:"
echo "  systemctl --user enable dms-awww.service"
echo "  systemctl --user start dms-awww.service"
echo ""
echo "To check status:"
echo "  systemctl --user status dms-awww.service"
echo ""
echo "NOTE: If WAYLAND_DISPLAY changes (e.g., after logout), update:"
echo "  $ENV_DIR/environment"
