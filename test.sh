#!/bin/bash
# dms-awww-integration Installation Verification Script
# This script checks that all components are properly installed and configured

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
PASS=0
FAIL=0
WARN=0

# Helper functions
pass() {
    echo -e "${GREEN}✓${NC} $1"
    ((PASS++)) || true
}

fail() {
    echo -e "${RED}✗${NC} $1"
    ((FAIL++)) || true
}

warn() {
    echo -e "${YELLOW}⚠${NC} $1"
    ((WARN++)) || true
}

info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

echo "======================================"
echo "DMS-AWWW Integration Test Suite"
echo "======================================"
echo ""

# Test 1: Check if watcher script exists and is executable
echo "[1] Checking watcher script..."
WATCHER="$HOME/.local/bin/dms-wallpaper-watcher"
if [ -f "$WATCHER" ]; then
    if [ -x "$WATCHER" ]; then
        pass "Watcher script exists and is executable: $WATCHER"
    else
        fail "Watcher script exists but is not executable: $WATCHER"
    fi
else
    fail "Watcher script not found: $WATCHER"
fi || true

# Test 2: Check if systemd service exists
echo ""
echo "[2] Checking systemd service..."
SERVICE="$HOME/.config/systemd/user/dms-wallpaper-watcher.service"
if [ -f "$SERVICE" ]; then
    pass "Service file exists: $SERVICE"
else
    fail "Service file not found: $SERVICE"
fi

# Test 3: Check if service is enabled
echo ""
echo "[3] Checking if service is enabled..."
if systemctl --user is-enabled dms-wallpaper-watcher.service &>/dev/null; then
    pass "Service is enabled"
else
    fail "Service is not enabled (run: systemctl --user enable dms-wallpaper-watcher.service)"
fi

# Test 4: Check if service is running
echo ""
echo "[4] Checking if service is running..."
if systemctl --user is-active dms-wallpaper-watcher.service &>/dev/null; then
    pass "Service is running"
else
    fail "Service is not running (run: systemctl --user start dms-wallpaper-watcher.service)"
fi

# Test 5: Check if DMS is installed
echo ""
echo "[5] Checking DMS installation..."
if command -v dms &>/dev/null; then
    pass "DMS is installed"
else
    fail "DMS is not found in PATH"
fi

# Test 6: Check if awww is installed
echo ""
echo "[6] Checking awww installation..."
if command -v awww &>/dev/null; then
    pass "awww is installed"
    AWWW_VERSION=$(awww --version 2>/dev/null || echo "unknown")
    info "  awww version: $AWWW_VERSION"
else
    fail "awww is not found in PATH (install from AUR: yay -S awww)"
fi

# Test 7: Check if DMS session file exists
echo ""
echo "[7] Checking DMS session file..."
SESSION_FILE="$HOME/.local/state/DankMaterialShell/session.json"
if [ -f "$SESSION_FILE" ]; then
    pass "DMS session file exists: $SESSION_FILE"
else
    fail "DMS session file not found: $SESSION_FILE"
fi

# Test 8: Check if log file exists and has recent entries
echo ""
echo "[8] Checking log file..."
LOG_FILE="/tmp/dms_wallpaper_watcher.log"
if [ -f "$LOG_FILE" ]; then
    pass "Log file exists: $LOG_FILE"
    # Check for recent entries (last 5 minutes)
    if find "$LOG_FILE" -mmin -5 &>/dev/null; then
        pass "Log file has recent activity (within 5 minutes)"
    else
        warn "Log file exists but no recent activity (may be normal if no wallpaper changes)"
    fi
    # Show last few lines
    info "  Last log entries:"
    tail -3 "$LOG_FILE" | sed 's/^/    /'
else
    warn "Log file not found: $LOG_FILE (will be created when service runs)"
fi

# Test 9: Check if DMS matugen command works
echo ""
echo "[9] Checking DMS matugen integration..."
if command -v dms &>/dev/null; then
    if dms matugen --help &>/dev/null; then
        pass "dms matugen command is available"
    else
        fail "dms matugen command failed"
    fi
else
    fail "dms command not found"
fi

# Test 10: Check Hyprland (for awww to work)
echo ""
echo "[10] Checking Wayland compositor..."
if [ -n "$WAYLAND_DISPLAY" ]; then
    pass "Wayland session detected (WAYLAND_DISPLAY=$WAYLAND_DISPLAY)"
else
    warn "Not running under Wayland (awww requires Hyprland)"
fi

if command -v hyprctl &>/dev/null; then
    pass "Hyprland is running"
else
    warn "hyprctl not found (awww requires Hyprland)"
fi

# Summary
echo ""
echo "======================================"
echo "Test Summary"
echo "======================================"
echo -e "${GREEN}Passed: $PASS${NC}"
if [ $WARN -gt 0 ]; then
    echo -e "${YELLOW}Warnings: $WARN${NC}"
fi
if [ $FAIL -gt 0 ]; then
    echo -e "${RED}Failed: $FAIL${NC}"
fi

if [ $FAIL -eq 0 ]; then
    echo ""
    echo -e "${GREEN}All critical tests passed!${NC}"
    exit 0
else
    echo ""
    echo -e "${RED}Some tests failed. Please fix the issues above.${NC}"
    exit 1
fi
