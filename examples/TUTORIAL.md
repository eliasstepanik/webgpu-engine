# Multi-Viewport Editor Tutorial

This tutorial will walk you through using the multi-viewport editor system, from basic panel operations to advanced multi-monitor workflows.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Basic Panel Operations](#basic-panel-operations)
3. [Layout Management](#layout-management)
4. [Multi-Window Workflows](#multi-window-workflows)
5. [Customization](#customization)
6. [Troubleshooting](#troubleshooting)

## Getting Started

### First Launch

1. **Build and run the editor**:
   ```bash
   cargo run
   ```

2. **Initial interface**: You'll see the default layout with four panels:
   - **Hierarchy** (left): Scene entity tree
   - **Viewport** (center): 3D scene view
   - **Inspector** (right): Entity properties
   - **Assets** (bottom): Asset browser

3. **Toggle modes**: Press `Tab` to switch between Editor UI mode and Game input mode.

### Understanding the Interface

#### Panel Components
- **Title Bar**: Shows panel name, contains detach button
- **Content Area**: Main panel functionality
- **Resize Handles**: Drag edges to resize panels
- **Context Menu**: Right-click title bar for options

#### Status Bar (Bottom)
Shows current scene, editor mode, entity count, and selection.

## Basic Panel Operations

### Panel Interaction

1. **Selecting Entities**:
   - Click entities in the Hierarchy panel
   - Properties appear in the Inspector panel
   - Selected entity is highlighted in Viewport

2. **Viewing Scene**:
   - Use mouse to navigate in Viewport
   - Camera controls work in Game input mode (Tab to toggle)

3. **Panel Resizing**:
   - Drag panel borders to resize
   - Double-click borders to auto-fit content

### Detaching Panels

**Method 1: Right-Click Menu** (Recommended)
1. Right-click on any panel's title bar
2. Select "Detach to Window" from context menu
3. Panel opens in new OS window

**Method 2: Manual Window Creation**
1. Panels automatically detach when requested through UI
2. Each detached panel gets independent window controls

### Reattaching Panels

**Method 1: Close Window**
1. Click X button on detached window
2. Panel automatically reattaches to main editor

**Method 2: Context Menu**
1. Right-click panel title in detached window
2. Select "Attach to Main Window"

## Layout Management

### Using Preset Layouts

#### Quick Layout Switching (Command Line)

**Windows:**
```batch
# Switch to developer layout
examples\switch_layout.bat developer

# Switch to dual monitor layout  
examples\switch_layout.bat dual_monitor

# See all available layouts
examples\switch_layout.bat
```

**Unix/Linux/macOS:**
```bash
# Switch to developer layout
./examples/switch_layout.sh developer

# Switch to dual monitor layout
./examples/switch_layout.sh dual_monitor

# See all available layouts
./examples/switch_layout.sh
```

**Python (Cross-platform):**
```bash
# List all layouts
python examples/switch_layout.py list

# Apply a layout
python examples/switch_layout.py apply developer

# Backup current layout
python examples/switch_layout.py backup
```

#### Available Preset Layouts

1. **Compact Layout** (`compact`)
   - Small panels for laptop screens
   - Efficient use of limited space
   - Resolution: 1366x768+

2. **Developer Layout** (`developer`)  
   - Balanced for general development
   - Good for coding and debugging
   - Resolution: 1920x1080+

3. **Artist Layout** (`artist`)
   - Large viewport for 3D work
   - Minimal UI clutter
   - Resolution: 1920x1080+

4. **Dual Monitor Layout** (`dual_monitor`)
   - Viewport detached to second monitor
   - Editor panels on primary monitor
   - Resolution: 3840x1080+ (dual 1920x1080)

5. **Minimal Layout** (`minimal`)
   - Essential panels only
   - Clean, distraction-free interface
   - Resolution: 1280x720+

6. **Ultrawide Layout** (`ultrawide`)
   - Horizontal panel arrangement
   - Takes advantage of wide screens
   - Resolution: 2560x1080+ or 3440x1440+

7. **Detached Workflow** (`detached`)
   - Most panels in separate windows
   - Maximum flexibility
   - Resolution: 1920x1080+ with good GPU

### Runtime Layout Management

#### Saving Your Layout
1. Arrange panels as desired
2. Go to `View → Save Layout` in menu bar
3. Layout saved to `editor_layout.json`

#### Loading Saved Layout
1. Go to `View → Load Layout` in menu bar
2. Layout restored from `editor_layout.json`

#### Resetting to Defaults
1. Go to `View → Reset Layout` in menu bar
2. All panels return to default positions
3. All detached windows are closed

### Automatic Layout Persistence
- Layout automatically saved when exiting editor
- Layout automatically loaded when starting editor
- No manual action required for basic persistence

## Multi-Window Workflows

### Single Monitor Optimization

**Layout Recommendations:**
- Use `developer` or `compact` layouts
- Keep all panels in main window
- Resize panels based on current task

**Workflow Tips:**
- Maximize viewport when doing 3D work
- Expand inspector when tweaking properties  
- Show/hide assets panel as needed

### Dual Monitor Setup

**Primary Monitor (Editor):**
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
|           (Detached)                |
|                                     |
+-------------------------------------+
```

**Setup Steps:**
1. Start with `dual_monitor` layout
2. Or manually detach viewport to second monitor
3. Position detached window on secondary monitor
4. Adjust sizes as needed

### Triple Monitor Workflow

**Monitor 1 (Primary - Code/Editor):**
- Hierarchy and Inspector panels
- Main editor interface

**Monitor 2 (Secondary - 3D View):**
- Detached Viewport (full screen)
- 3D scene navigation

**Monitor 3 (Tertiary - Assets):**
- Detached Assets panel
- Reference materials, documentation

### Multi-Window Best Practices

1. **Performance Considerations:**
   - Limit to 4 detached windows max
   - Monitor GPU memory usage
   - Close unused detached windows

2. **Organization Tips:**
   - Group related panels on same monitor
   - Keep frequently used panels easily accessible
   - Use consistent window positions

3. **Workflow Efficiency:**
   - Save multiple layout configurations
   - Switch layouts based on current task
   - Use keyboard shortcuts when available

## Customization

### Manual Layout Editing

Edit `editor_layout.json` directly for precise control:

```json
{
  "id": "hierarchy",
  "title": "Custom Hierarchy",
  "position": [10.0, 50.0],
  "size": [300.0, 400.0],
  "was_detached": false,
  "is_visible": true
}
```

**Parameters:**
- `position`: [x, y] in pixels from top-left
- `size`: [width, height] in pixels
- `was_detached`: Whether panel was detached when saved
- `is_visible`: Panel visibility (true/false)

### Creating Custom Layouts

1. **Arrange panels** as desired in editor
2. **Save layout** using `View → Save Layout`
3. **Copy file** to create variations:
   ```bash
   cp editor_layout.json examples/my_custom_layout.json
   ```
4. **Edit as needed** with text editor
5. **Load by copying back**:
   ```bash
   cp examples/my_custom_layout.json editor_layout.json
   ```

### Panel Customization

#### Changing Panel Titles
Edit the `title` field in layout JSON:
```json
"title": "My Custom Hierarchy Panel"
```

#### Default Panel Sizes
Modify `size` array for default dimensions:
```json
"size": [400.0, 600.0]
```

#### Starting Positions
Adjust `position` for initial placement:
```json
"position": [50.0, 100.0]
```

### Sharing Layouts

#### Team Collaboration
1. **Standardize layouts**: Agree on common layouts for team
2. **Version control**: Include layout files in project repository
3. **Documentation**: Document layout purposes and use cases

#### Export/Import Process
```bash
# Export current layout
cp editor_layout.json shared_layouts/team_standard.json

# Import shared layout
cp shared_layouts/team_standard.json editor_layout.json
```

## Troubleshooting

### Common Issues

#### Panel Won't Detach
**Symptoms:** Right-click menu doesn't show detach option
**Solutions:**
- Ensure detached window manager is initialized
- Check console for error messages
- Restart editor and try again
- Verify GPU has sufficient memory

#### Layout Not Saving
**Symptoms:** Changes lost when restarting editor
**Solutions:**
- Check file permissions on project directory
- Ensure `editor_layout.json` is writable
- Look for error messages in View menu
- Try manual save via View menu

#### Performance Issues
**Symptoms:** Low FPS, stuttering with multiple windows
**Solutions:**
- Reduce number of detached windows
- Close unused detached panels
- Monitor GPU memory usage
- Lower graphics settings if needed

#### Window Management Problems
**Symptoms:** Windows not closing, panels not reattaching
**Solutions:**
- Use View → Reset Layout to recover
- Check console for cleanup errors
- Restart editor to clear state
- Manually close stuck windows

### Performance Optimization

#### GPU Memory Management
- Each detached window uses ~50-100MB GPU memory
- Monitor usage with GPU-Z or similar tools
- Close detached windows when not needed

#### CPU Usage Optimization
- Each window adds ~5-10% CPU overhead
- Limit simultaneous detached windows
- Use efficient layouts for your hardware

#### Recommended Limits
- **Low-end systems**: Max 2 detached windows
- **Mid-range systems**: Max 3-4 detached windows  
- **High-end systems**: 4+ detached windows OK

### Debug Information

#### Console Logging
Enable detailed logging:
```bash
RUST_LOG=debug cargo run
```

Look for messages containing:
- `DetachedWindowManager`
- `PanelManager`
- `surface` (for GPU issues)

#### Common Error Messages

**"Surface does not exist"**
- GPU adapter issue
- Restart application
- Check graphics drivers

**"Failed to create window"**
- OS window limit reached
- Close other applications
- Reduce detached windows

**"Layout file not found"**
- Missing layout file
- Use View → Reset Layout
- Copy from examples directory

## Advanced Tips

### Keyboard Shortcuts
- `Tab`: Toggle UI/Game input mode
- `Ctrl+N`: New scene
- `Ctrl+O`: Open scene
- `Ctrl+S`: Save scene
- `Ctrl+Shift+S`: Save scene as

### Scene Management
- Scenes automatically include layout information
- Use different layouts for different types of scenes
- Save scene-specific layouts as needed

### Development Workflow
1. **Planning Phase**: Use minimal layout for focus
2. **Development Phase**: Use developer layout for coding
3. **Testing Phase**: Use artist layout for visual verification
4. **Polish Phase**: Use dual monitor for detailed work

### Content Creation Workflow  
1. **Concept Phase**: Use minimal layout
2. **Modeling Phase**: Use artist layout with large viewport
3. **Texturing Phase**: Use dual monitor with assets detached
4. **Final Review**: Use detached workflow for comprehensive view

This completes the comprehensive tutorial for the multi-viewport editor system. Experiment with different layouts and workflows to find what works best for your specific needs!