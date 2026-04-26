#!/usr/bin/env bash
set -euo pipefail

echo "========================================"
echo "Building Event Timers for GW2"
echo "========================================"

cargo xwin build --release --target x86_64-pc-windows-msvc
ADDONS_DIR="/mnt/ssd/Games/Guild Wars 2/addons"

cp target/x86_64-pc-windows-msvc/release/event_timers.dll \
   "$ADDONS_DIR/"
echo "Build and copy complete."
