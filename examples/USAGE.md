# Multi-Viewport Editor Usage Guide

## Overview

The multi-viewport editor system allows you to detach editor panels into separate OS windows, enabling:
- Multi-monitor workflows
- Flexible panel arrangements
- Persistent layout configurations
- Independent window management

## Basic Operations

### Detaching Panels

To detach a panel into its own window:

1. **Using Right-Click Menu** (Recommended):
   - Right-click on any panel's title bar
   - Select "Detach to Window" from the context menu
   - The panel will open in a new OS window

2. **Using Keyboard Shortcut**:
   - Focus on a panel
   - Press `Ctrl+D` to detach (if implemented)

### Reattaching Panels

To reattach a detached panel back to the main window:

1. **Close the Window**:
   - Click the X button on the detached window
   - The panel will automatically reattach to the main editor

2. **Using Right-Click Menu**:
   - Right-click on the panel title in the detached window
   - Select "Attach to Main Window"

### Layout Management

#### Saving Layouts
- **Auto-save**: Layouts are automatically saved when you exit the editor
- **Manual save**: Use `View → Save Layout` to save current arrangement
- **File location**: Saved as `editor_layout.json` in the project directory

#### Loading Layouts
- **Auto-load**: Layouts are automatically loaded when starting the editor
- **Manual load**: Use `View → Load Layout` to reload the saved layout
- **Reset**: Use `View → Reset Layout` to return to defaults

#### Custom Layout Files
You can create custom layout files and load them manually:

```bash
# Copy a layout to the default location
cp examples/developer_layout.json editor_layout.json

# Or create your own custom layout
# Edit editor_layout.json manually after saving your preferred arrangement
```

## Panel Types

### Hierarchy Panel
- **Purpose**: Shows the scene entity tree
- **Features**: 
  - Expandable/collapsible entity hierarchy
  - Entity selection
  - Parent-child relationships
- **Typical Usage**: Keep docked on the left side or detached on secondary monitor

### Inspector Panel  
- **Purpose**: Displays properties of the selected entity
- **Features**:
  - Component editing
  - Transform manipulation
  - Material properties
- **Typical Usage**: Keep docked on the right side for easy access

### Viewport Panel
- **Purpose**: 3D scene rendering and navigation
- **Features**:
  - Real-time 3D view
  - Camera controls
  - Object selection
- **Typical Usage**: Center of main window or full-screen on secondary monitor

### Assets Panel
- **Purpose**: Asset browser and management
- **Features**:
  - File browser
  - Asset preview
  - Import/export tools
- **Typical Usage**: Bottom of screen or detached for easy access

## Workflow Examples

### Single Monitor Development
```
+------------------+------------------+
|    Hierarchy     |    Inspector     |
|                  |                  |
+------------------+------------------+
|            Viewport                 |
|                                     |
|                                     |
+-------------------------------------+
|             Assets                  |
+-------------------------------------+
```

### Dual Monitor Setup
**Primary Monitor (Editor UI):**
```
+------------------+------------------+
|    Hierarchy     |    Inspector     |
|                  |                  |
+------------------+------------------+
|             Assets                  |
+-------------------------------------+
```

**Secondary Monitor (Content):**
```
+-------------------------------------+
|                                     |
|            Viewport                 |
|           (Full Screen)             |
|                                     |
+-------------------------------------+
```

### Artist Workflow
- **Main Window**: Viewport maximized for 3D work
- **Detached Windows**: 
  - Inspector on secondary monitor for tweaking properties
  - Assets panel for quick asset access
  - Hierarchy minimal or hidden

### Programmer Workflow
- **Main Window**: Balanced layout with all panels visible
- **Detached Windows**:
  - Viewport on secondary monitor for testing
  - Inspector easily accessible for debugging

## Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| Toggle UI/Game Mode | `Tab` |
| New Scene | `Ctrl+N` |
| Open Scene | `Ctrl+O` |
| Save Scene | `Ctrl+S` |
| Save Scene As | `Ctrl+Shift+S` |

## Troubleshooting

### Panel Not Detaching
- Ensure you're right-clicking on the panel title bar
- Check that the detached window manager is initialized
- Look for error messages in the console

### Layout Not Saving
- Check file permissions in the project directory
- Ensure `editor_layout.json` is writable
- Look for error messages in View menu operations

### Window Management Issues
- **Windows not closing properly**: Check console for cleanup errors
- **Panels not reattaching**: Try manually using View → Reset Layout
- **Performance issues**: Limit number of detached windows (recommend max 4)

### Performance Tips
- **GPU Memory**: Each detached window uses additional GPU memory
- **CPU Usage**: More windows = more render calls
- **Recommended Limits**: 
  - Max 4 detached windows for optimal performance
  - Use viewport detachment sparingly on lower-end systems

## Advanced Configuration

### Manual Layout Editing

You can manually edit layout JSON files for precise control:

```json
{
  "id": "hierarchy",
  "title": "Scene Hierarchy",
  "position": [10.0, 50.0],
  "size": [300.0, 400.0],
  "was_detached": false,
  "is_visible": true
}
```

- **position**: [x, y] coordinates in pixels
- **size**: [width, height] in pixels  
- **was_detached**: Whether panel was detached when saved
- **is_visible**: Panel visibility state

### Creating Custom Layouts

1. Arrange panels as desired in the editor
2. Use `View → Save Layout` to create the JSON file
3. Copy the file to create variations:
   ```bash
   cp editor_layout.json examples/my_custom_layout.json
   ```
4. Edit the copied file as needed
5. Load by copying back:
   ```bash
   cp examples/my_custom_layout.json editor_layout.json
   ```

## Tips and Best Practices

### Monitor Setup
- **Primary Monitor**: Keep main editor window with frequently used panels
- **Secondary Monitor**: Use for viewport or secondary workflow panels
- **Ultra-wide**: Take advantage of horizontal space for side-by-side panels

### Workflow Optimization
- **Save multiple layouts**: Create different layouts for different tasks
- **Start with defaults**: Use reset layout when trying new arrangements
- **Test performance**: Monitor FPS when using multiple detached windows

### Panel Organization
- **Group related panels**: Keep inspector near hierarchy for entity work
- **Maximize viewport**: Give 3D view the most screen real estate when possible
- **Keep assets accessible**: Place assets panel where you can easily reach it

### Sharing Layouts
- **Version control**: Include layout files in your project repository
- **Team standards**: Agree on common layouts for collaborative work
- **Document setups**: Create README files explaining layout purposes