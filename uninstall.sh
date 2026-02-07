#!/bin/bash
set -e

INSTALL_DIR="$HOME/.cargo/bin"
USER_SYSTEMD_DIR="$HOME/.config/systemd/user"

echo "Stopping and disabling service..."
systemctl --user stop dms-awww.service 2>/dev/null || true
systemctl --user disable dms-awww.service 2>/dev/null || true

echo "Removing systemd service..."
rm -f "$USER_SYSTEMD_DIR/dms-awww.service"

echo "Reloading systemd daemon..."
systemctl --user daemon-reload
systemctl --user reset-failed 2>/dev/null || true

echo "Removing binary..."
rm -f "$INSTALL_DIR/dms-awww"

echo ""
echo "Uninstall complete!"
