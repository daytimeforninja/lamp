#!/usr/bin/env bash
set -euo pipefail

# refresh.sh — rebuild desktop + mobile, launch desktop, push mobile via adb
#
# Usage:
#   ./scripts/refresh.sh          # build both, launch desktop, push mobile
#   ./scripts/refresh.sh desktop   # desktop only
#   ./scripts/refresh.sh mobile    # mobile only

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m'

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MOBILE_DIR="$ROOT/lamp-mobile"
APK_PATH="$MOBILE_DIR/app/build/outputs/apk/debug/app-debug.apk"
DESKTOP_BIN="$ROOT/target/debug/lamp"
MOBILE_PKG="com.lamp.mobile"

mode="${1:-all}"

build_desktop() {
    echo -e "${YELLOW}Building desktop...${NC}"
    cd "$ROOT"
    cargo build --bin lamp 2>&1
    echo -e "${GREEN}Desktop build OK${NC}"
}

launch_desktop() {
    echo -e "${YELLOW}Launching desktop...${NC}"
    # Kill existing instance (match the exact binary path to avoid killing cargo)
    if pgrep -f "$DESKTOP_BIN" >/dev/null 2>&1; then
        pkill -f "$DESKTOP_BIN" 2>/dev/null || true
        sleep 1
    fi
    "$DESKTOP_BIN" &>/dev/null &
    disown
    echo -e "${GREEN}Desktop launched (PID $!)${NC}"
}

build_mobile() {
    echo -e "${YELLOW}Building mobile...${NC}"
    cd "$MOBILE_DIR"
    nix develop -c gradle assembleDebug 2>&1
    echo -e "${GREEN}Mobile build OK${NC}"
}

push_mobile() {
    if ! adb devices 2>/dev/null | grep -q "device$"; then
        echo -e "${RED}No adb device connected${NC}"
        return 1
    fi
    echo -e "${YELLOW}Installing APK via adb...${NC}"
    adb install -r "$APK_PATH" 2>&1
    echo -e "${GREEN}APK installed${NC}"
    # Launch the app
    adb shell am start -n "$MOBILE_PKG/.MainActivity" 2>/dev/null || true
    echo -e "${GREEN}Mobile app launched${NC}"
}

case "$mode" in
    desktop)
        build_desktop
        launch_desktop
        ;;
    mobile)
        build_mobile
        push_mobile
        ;;
    all)
        build_desktop
        launch_desktop
        build_mobile
        push_mobile
        ;;
    *)
        echo "Usage: $0 [desktop|mobile|all]"
        exit 1
        ;;
esac

echo -e "${GREEN}Done!${NC}"
