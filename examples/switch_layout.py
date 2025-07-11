#!/usr/bin/env python3
"""
Layout Switcher for Multi-Viewport Editor

This script helps you easily switch between different layout presets.
"""

import json
import shutil
import sys
from pathlib import Path

def load_config():
    """Load the layout configuration file."""
    config_path = Path(__file__).parent / "layout_config.json"
    try:
        with open(config_path, 'r') as f:
            return json.load(f)
    except FileNotFoundError:
        print(f"Error: Configuration file not found at {config_path}")
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"Error: Invalid JSON in configuration file: {e}")
        sys.exit(1)

def list_layouts(config):
    """Display all available layouts."""
    print("Available layouts:")
    print("=" * 50)
    
    for layout_id, layout_info in config["layouts"].items():
        print(f"\n{layout_id}:")
        print(f"  Name: {layout_info['name']}")
        print(f"  Description: {layout_info['description']}")
        print(f"  Recommended Resolution: {layout_info['recommended_resolution']}")
        print(f"  Use Cases: {', '.join(layout_info['use_cases'])}")

def apply_layout(config, layout_id):
    """Apply a specific layout."""
    if layout_id not in config["layouts"]:
        print(f"Error: Layout '{layout_id}' not found.")
        print("Use 'list' command to see available layouts.")
        sys.exit(1)
    
    layout_info = config["layouts"][layout_id]
    source_file = Path(__file__).parent / layout_info["file"]
    target_file = Path(__file__).parent.parent / "editor_layout.json"
    
    if not source_file.exists():
        print(f"Error: Layout file '{source_file}' not found.")
        sys.exit(1)
    
    try:
        shutil.copy2(source_file, target_file)
        print(f"✓ Applied layout: {layout_info['name']}")
        print(f"  Description: {layout_info['description']}")
        print(f"  File: {layout_info['file']} -> editor_layout.json")
        print("\nRestart the editor to see the new layout.")
    except Exception as e:
        print(f"Error: Failed to copy layout file: {e}")
        sys.exit(1)

def backup_current_layout():
    """Backup the current layout."""
    current_layout = Path(__file__).parent.parent / "editor_layout.json"
    backup_file = Path(__file__).parent / "current_layout_backup.json"
    
    if current_layout.exists():
        try:
            shutil.copy2(current_layout, backup_file)
            print(f"✓ Current layout backed up to: {backup_file}")
        except Exception as e:
            print(f"Warning: Failed to backup current layout: {e}")
    else:
        print("No current layout file to backup.")

def restore_backup():
    """Restore the backed up layout."""
    backup_file = Path(__file__).parent / "current_layout_backup.json"
    target_file = Path(__file__).parent.parent / "editor_layout.json"
    
    if not backup_file.exists():
        print("Error: No backup file found.")
        sys.exit(1)
    
    try:
        shutil.copy2(backup_file, target_file)
        print("✓ Backup layout restored.")
    except Exception as e:
        print(f"Error: Failed to restore backup: {e}")
        sys.exit(1)

def show_usage():
    """Show usage information."""
    print("Multi-Viewport Editor Layout Switcher")
    print("=" * 40)
    print("\nUsage:")
    print("  python switch_layout.py <command> [layout_id]")
    print("\nCommands:")
    print("  list                 - Show all available layouts")
    print("  apply <layout_id>    - Apply a specific layout")
    print("  backup               - Backup current layout")
    print("  restore              - Restore backed up layout")
    print("  help                 - Show this help message")
    print("\nExamples:")
    print("  python switch_layout.py list")
    print("  python switch_layout.py apply developer")
    print("  python switch_layout.py apply dual_monitor")

def main():
    """Main entry point."""
    if len(sys.argv) < 2:
        show_usage()
        sys.exit(1)
    
    command = sys.argv[1].lower()
    config = load_config()
    
    if command == "list":
        list_layouts(config)
    elif command == "apply":
        if len(sys.argv) < 3:
            print("Error: Please specify a layout ID.")
            print("Use 'list' command to see available layouts.")
            sys.exit(1)
        layout_id = sys.argv[2]
        backup_current_layout()
        apply_layout(config, layout_id)
    elif command == "backup":
        backup_current_layout()
    elif command == "restore":
        restore_backup()
    elif command in ["help", "-h", "--help"]:
        show_usage()
    else:
        print(f"Error: Unknown command '{command}'")
        show_usage()
        sys.exit(1)

if __name__ == "__main__":
    main()