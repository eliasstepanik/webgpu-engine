# Multi-Viewport Editor Examples

⚠️ **Important Notice**: Multi-window detachment is temporarily disabled due to imgui-rs 0.12 limitations. All panels currently remain in the main window. See `../MULTI_WINDOW_LIMITATION.md` for details.

This directory contains example files and configurations for the editor panel layout system.

## Files Overview

- **`compact_layout.json`** - A compact layout with smaller panels for smaller screens
- **`developer_layout.json`** - Layout focused on development workflow (recommended)
- **`artist_layout.json`** - Layout optimized for content creation
- **`minimal_layout.json`** - Minimal layout with only essential panels
- **`ultrawide_layout.json`** - Layout optimized for ultrawide monitors
- **`demo_scene.json`** - Example scene file for testing
- **`USAGE.md`** - Detailed usage instructions

## Quick Start

1. **Copy a layout file** to the project root as `editor_layout.json`:
   ```bash
   cp examples/developer_layout.json editor_layout.json
   ```

2. **Run the editor**:
   ```bash
   cargo run
   ```

3. **Use the multi-viewport features**:
   - Right-click on panel titles to detach/attach windows
   - Use View menu to save/load/reset layouts
   - Drag panels to reposition them within windows
   - Close detached windows to reattach panels

## Layout Features

### Detachable Panels
- **Hierarchy Panel**: Scene entity tree view
- **Inspector Panel**: Selected entity properties
- **Viewport Panel**: 3D scene view
- **Assets Panel**: Asset browser and management

### Persistence
- Layouts are automatically saved on exit
- Manual save/load through View menu
- JSON format for easy sharing and editing

### Multi-Window Support
- Each detached panel gets its own OS window
- Independent ImGui contexts per window
- Proper window lifecycle management
- Thread-safe state synchronization

## Customization

You can manually edit layout JSON files to customize:
- Panel positions and sizes
- Default visibility states
- Panel titles
- Which panels start detached

See the example files for reference on the JSON structure.