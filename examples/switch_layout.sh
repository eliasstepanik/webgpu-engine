#!/bin/bash
# Unix shell script for layout switching

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
TARGET_FILE="$PROJECT_DIR/editor_layout.json"

show_usage() {
    echo "Multi-Viewport Editor Layout Switcher"
    echo "====================================="
    echo
    echo "Usage: $0 <layout_name>"
    echo
    echo "Available layouts:"
    echo "  compact      - Compact layout for small screens"
    echo "  developer    - Balanced layout for development"
    echo "  artist       - Viewport-focused for content creation"
    echo "  dual_monitor - Optimized for dual monitor setups"
    echo "  minimal      - Clean interface with essentials only"
    echo "  ultrawide    - Optimized for ultrawide monitors"
    echo "  detached     - Multi-window workflow"
    echo
    echo "Examples:"
    echo "  $0 developer"
    echo "  $0 dual_monitor"
}

backup_current() {
    if [[ -f "$TARGET_FILE" ]]; then
        cp "$TARGET_FILE" "$SCRIPT_DIR/current_layout_backup.json"
        echo "✓ Current layout backed up"
    fi
}

if [[ $# -eq 0 ]]; then
    show_usage
    exit 0
fi

LAYOUT="$1"

# Backup current layout
backup_current

# Determine source file
case "$LAYOUT" in
    "compact")
        SOURCE_FILE="$SCRIPT_DIR/compact_layout.json"
        ;;
    "developer")
        SOURCE_FILE="$SCRIPT_DIR/developer_layout.json"
        ;;
    "artist")
        SOURCE_FILE="$SCRIPT_DIR/artist_layout.json"
        ;;
    "dual_monitor")
        SOURCE_FILE="$SCRIPT_DIR/dual_monitor_layout.json"
        ;;
    "minimal")
        SOURCE_FILE="$SCRIPT_DIR/minimal_layout.json"
        ;;
    "ultrawide")
        SOURCE_FILE="$SCRIPT_DIR/ultrawide_layout.json"
        ;;
    "detached")
        SOURCE_FILE="$SCRIPT_DIR/detached_workflow.json"
        ;;
    *)
        echo "Error: Unknown layout '$LAYOUT'"
        echo "Use '$0' without arguments to see available layouts."
        exit 1
        ;;
esac

if [[ ! -f "$SOURCE_FILE" ]]; then
    echo "Error: Layout file '$SOURCE_FILE' not found."
    exit 1
fi

# Copy the layout file
cp "$SOURCE_FILE" "$TARGET_FILE"

echo "✓ Applied layout: $LAYOUT"
echo "  File: $SOURCE_FILE"
echo "  Target: $TARGET_FILE"
echo
echo "Restart the editor to see the new layout."