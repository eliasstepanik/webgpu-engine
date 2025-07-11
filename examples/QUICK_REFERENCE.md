# Multi-Viewport Editor Quick Reference

## Essential Operations

| Action | Method |
|--------|--------|
| **Detach Panel** | Right-click title → "Detach to Window" |
| **Reattach Panel** | Close detached window |
| **Save Layout** | View → Save Layout |
| **Load Layout** | View → Load Layout |
| **Reset Layout** | View → Reset Layout |
| **Toggle UI Mode** | Press `Tab` |

## Quick Layout Switching

```bash
# Windows
examples\switch_layout.bat <layout_name>

# Unix/Linux/macOS  
./examples/switch_layout.sh <layout_name>

# Python (cross-platform)
python examples/switch_layout.py apply <layout_name>
```

## Available Layouts

| Layout | Description | Best For |
|--------|-------------|----------|
| `compact` | Small panels for laptops | Limited screen space |
| `developer` | Balanced for development | General coding work |
| `artist` | Large viewport focus | 3D content creation |
| `dual_monitor` | Viewport on 2nd monitor | Dual monitor setups |
| `minimal` | Essential panels only | Focused work |
| `ultrawide` | Horizontal arrangement | Ultrawide monitors |
| `detached` | Multi-window workflow | Power users |

## Panel Types

| Panel | Purpose | Typical Size |
|-------|---------|-------------|
| **Hierarchy** | Scene entity tree | 250-350px wide |
| **Inspector** | Entity properties | 300-400px wide |
| **Viewport** | 3D scene view | 600-1200px wide |
| **Assets** | File browser | 400-800px wide |

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Tab` | Toggle UI/Game mode |
| `Ctrl+N` | New scene |
| `Ctrl+O` | Open scene |
| `Ctrl+S` | Save scene |
| `Ctrl+Shift+S` | Save scene as |

## Performance Guidelines

| System | Max Detached Windows | Notes |
|--------|---------------------|-------|
| **Low-end** | 2 | Basic functionality |
| **Mid-range** | 3-4 | Good performance |
| **High-end** | 4+ | Full features |

## File Locations

| File | Purpose |
|------|---------|
| `editor_layout.json` | Current active layout |
| `examples/*.json` | Preset layout files |
| `examples/layout_config.json` | Layout configuration |

## Troubleshooting

| Issue | Quick Fix |
|-------|----------|
| **Panel won't detach** | Restart editor |
| **Layout not saving** | Check file permissions |
| **Performance issues** | Reduce detached windows |
| **Windows not closing** | Use View → Reset Layout |

## Common Workflows

### Single Monitor
```
+----------+----------+
| Hier.    | Insp.    |
+----------+----------+
|    Viewport         |
+---------------------+
|      Assets         |
+---------------------+
```

### Dual Monitor
**Monitor 1:** Hierarchy + Inspector + Assets  
**Monitor 2:** Viewport (detached)

### Development
1. Start with `developer` layout
2. Detach viewport when testing
3. Use inspector for debugging
4. Save custom variations

### Art Creation
1. Start with `artist` layout  
2. Maximize viewport space
3. Detach assets for reference
4. Use minimal inspector