name: "Create Comprehensive README Documentation"
description: |

## Purpose
Create a professional, comprehensive README.md that serves as the primary entry point for developers discovering the WebGPU engine. The README should follow Rust game engine documentation best practices, showcase the project's capabilities, and provide clear onboarding instructions.

## Core Principles
1. **Progressive Disclosure**: Brief intro → Key features → Quick start → Detailed docs
2. **Visual Appeal**: Use badges, tables, code examples, and ASCII diagrams
3. **Developer-Focused**: Prioritize getting developers running quickly
4. **Cross-Reference**: Link to existing detailed documentation
5. **Rust Conventions**: Follow rustdoc and crates.io best practices

---

## Goal
Replace the empty README.md with a comprehensive documentation that:
- Immediately communicates what the engine does and why it's valuable
- Gets developers from zero to running in under 5 minutes
- Showcases the multi-viewport editor and Rhai scripting capabilities
- Provides clear paths to deeper documentation
- Prepares for eventual crates.io publishing

## Why
- **First Impressions**: The README is often the only documentation developers read
- **Discoverability**: A good README improves GitHub searchability and engagement
- **Onboarding**: Reduces friction for new contributors and users
- **Professional Image**: Demonstrates project maturity and maintenance quality

## What
A README.md file structured with:
1. Project title with badges
2. Brief tagline and hero features
3. Quick start section (< 5 steps)
4. Architecture overview with ASCII diagram
5. Feature showcase with subsections
6. Development guide
7. Examples and demos
8. Contributing guidelines
9. License and credits

### Success Criteria
- [ ] Developer can go from clone to running engine in < 5 minutes
- [ ] All build commands work as documented
- [ ] Links to deeper documentation are correct
- [ ] README renders properly on GitHub
- [ ] Integrates with Cargo.toml for crates.io

## All Needed Context

### Documentation & References
```yaml
# MUST READ - Include these in your context window
- url: https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html
  why: Rust documentation conventions - brief intro → details → examples pattern
  
- url: https://doc.rust-lang.org/cargo/reference/manifest.html#the-readme-field
  why: Integration with Cargo.toml readme field for crates.io
  
- file: examples/README.md
  why: Project's documentation style - clear sections, code blocks, quick start pattern
  
- file: examples/QUICK_REFERENCE.md
  why: Table formatting style for shortcuts and commands
  
- file: examples/TUTORIAL.md
  why: Step-by-step instruction style and troubleshooting sections
  
- file: game/assets/scenes/README.md
  why: JSON example formatting and API usage examples
  
- file: PLANNING.md
  why: Architecture overview, module descriptions, build modes table
  
- file: justfile
  why: All available build commands and their descriptions
  
- file: .claude/documentation/viewport-debugging-guide.md
  why: Technical documentation style with problem/solution format

- url: https://users.rust-lang.org/t/best-practice-for-documenting-crates-readme-md-vs-documentation-comments/124254
  why: Best practices for README vs rustdoc comments
```

### Current Codebase Tree
```bash
.
├── README.md                 # Currently empty - to be replaced
├── PLANNING.md              # Architecture documentation  
├── Cargo.toml               # Workspace definition
├── justfile                 # Build commands
├── assets/                  # Global assets
│   └── scripts/            # Example Rhai scripts
├── engine/                  # Core engine library
│   ├── Cargo.toml
│   └── src/
│       ├── core/           # ECS, transforms, camera
│       ├── graphics/       # WebGPU renderer
│       ├── io/            # Scene serialization
│       ├── scripting/     # Rhai integration
│       └── windowing/     # Multi-window support
├── editor/                  # ImGui-based editor
│   ├── Cargo.toml
│   └── src/
├── game/                    # Game binary
│   ├── Cargo.toml
│   ├── src/
│   └── assets/
│       ├── scenes/        # Demo scenes
│       └── scripts/       # Game scripts
└── examples/               # Layout examples & docs
    ├── README.md
    ├── TUTORIAL.md
    ├── QUICK_REFERENCE.md
    └── *_layout.json      # Editor layouts
```

### Known Gotchas & Project Specifics
```markdown
# CRITICAL: WebGPU requires modern GPU drivers
# The engine will panic without proper WebGPU support

# CRITICAL: Editor is only included in dev builds by default
# Production builds use --no-default-features to exclude it

# GOTCHA: Multi-window detachment temporarily disabled
# Due to imgui-rs 0.12 limitations (see examples/README.md warning)

# PATTERN: Project uses 'just' command runner
# All commands should reference justfile, not raw cargo commands

# GOTCHA: Custom imgui fork in Cargo.toml patch section
# Links to github.com/eliasstepanik/imgui-rs

# CONVENTION: Badges should include:
# - Build status (when CI is added)
# - Crates.io version (when published)  
# - Docs.rs link (when published)
# - License badge (MIT/Apache)

# STYLE: Use collapsible sections for long content
# <details><summary>Click to expand</summary>content</details>

# REQUIREMENT: README must work with Cargo.toml readme field
# Path relative to Cargo.toml root
```

## Implementation Blueprint

### README Structure
```markdown
# WebGPU Engine 🎮

[![License](badge-url)](license-url)
[![Rust](badge-url)](rust-version)
[Future: crates.io, docs.rs, build status]

> A modern, modular 3D game engine built with Rust and WebGPU, featuring a sophisticated multi-viewport editor and Rhai scripting.

## ✨ Features

- 🚀 **Modern Rendering** - WebGPU-based pipeline with depth testing and per-object transforms
- 🎯 **Entity Component System** - Efficient ECS using hecs with transform hierarchy
- 🖼️ **Multi-Viewport Editor** - Detachable ImGui panels for flexible workflows
- 📜 **Rhai Scripting** - Hot-reloadable scripts with property persistence
- 📦 **Scene System** - JSON-based scene serialization with asset hot-reloading
- 🏗️ **Modular Architecture** - Clean separation of engine, editor, and game

## 🚀 Quick Start

[Prerequisites and 3-4 step quick start]

## 🏛️ Architecture

[ASCII diagram from PLANNING.md]

## 📚 Documentation

[Links to detailed docs]

## 🎮 Examples

[Code examples and demo descriptions]

## 🛠️ Development

[Build modes, testing, contributing]

## 📄 License

[License information]
```

### List of Tasks

```yaml
Task 1 - Create README Header:
CREATE README.md:
  - Project title with emoji
  - Badge placeholders with TODOs for future CI
  - Compelling tagline
  - Feature list with emojis and bold keywords
  - PATTERN: Follow examples/README.md header style

Task 2 - Write Quick Start Section:
ADD to README.md:
  - Prerequisites (Rust version, GPU requirements)
  - Clone command with repo URL
  - Simple "just run" command
  - What user should see when it works
  - Link to troubleshooting
  - PATTERN: < 5 steps like examples/README.md

Task 3 - Add Architecture Overview:
ADD to README.md:
  - ASCII diagram of three-crate structure
  - Brief description of each module
  - Table of crate responsibilities from PLANNING.md
  - Link to full PLANNING.md
  - PATTERN: Use tables like QUICK_REFERENCE.md

Task 4 - Document Key Features:
ADD to README.md:
  - Rendering capabilities subsection
  - ECS and transform hierarchy explanation
  - Editor features (with note about temporary limitation)
  - Scripting system with Rhai example
  - Scene system with JSON snippet
  - PATTERN: Use code blocks like game/assets/scenes/README.md

Task 5 - Add Development Guide:
ADD to README.md:
  - Build modes table (dev vs prod)
  - All just commands with descriptions
  - Testing instructions
  - Project structure overview
  - PATTERN: Tables from PLANNING.md

Task 6 - Include Examples Section:
ADD to README.md:
  - How to run example scenes
  - Editor layout options
  - Script examples
  - Links to examples/ directory
  - PATTERN: Reference style from examples/TUTORIAL.md

Task 7 - Add Contributing & License:
ADD to README.md:
  - Link to CLAUDE.md for AI guidelines
  - Basic contributing guidelines
  - Code style requirements
  - License section (check Cargo.toml)
  - Credits and acknowledgments

Task 8 - Polish and Validate:
MODIFY README.md:
  - Add table of contents if > 500 lines
  - Ensure all links work
  - Check markdown rendering
  - Verify code examples are accurate
  - Add collapsible sections for long content
```

### Integration Points
```yaml
CARGO:
  - ensure: README.md path is correct in root Cargo.toml
  - note: May need readme = "README.md" field
  
GITHUB:
  - renders: Automatically on repository main page
  - relative: All links must work from GitHub web view
  
EXAMPLES:
  - reference: Link to examples/ for layouts
  - highlight: Multi-viewport editor as key feature
  
DOCUMENTATION:
  - cross-link: PLANNING.md for architecture
  - cross-link: examples/TUTORIAL.md for detailed guide
  - cross-link: CLAUDE.md for AI development
```

## Validation Loop

### Level 1: Markdown Validation
```bash
# Check markdown syntax (if markdownlint installed)
markdownlint README.md

# Preview in terminal (if glow installed)
glow README.md

# Expected: Clean formatting, no broken links
```

### Level 2: Content Validation
```bash
# Verify all commands work
just run  # Should start the engine

# Test quick start steps
git clone https://github.com/eliasstepanik/webgpu-engine.git test-readme
cd test-readme
just run

# Expected: Engine runs with editor visible
```

### Level 3: Link Validation
```bash
# Check all internal links
grep -oP '(?<=\[)[^\]]+(?=\]\([^)]+\))' README.md | while read -r link; do
  file=$(grep -oP "(?<=$link\]\()[^)#]+" README.md)
  if [[ -f "$file" ]]; then
    echo "✓ $file"
  else
    echo "✗ $file (missing)"
  fi
done

# Expected: All linked files exist
```

## Final Validation Checklist
- [ ] README renders correctly on GitHub
- [ ] Quick start works for new user
- [ ] All `just` commands are documented correctly
- [ ] Architecture diagram displays properly
- [ ] Code examples are syntax highlighted
- [ ] Links to other docs work
- [ ] No placeholder content remains
- [ ] Badges have correct URLs (or TODO comments)
- [ ] Integrates with Cargo.toml if readme field exists

---

## Anti-Patterns to Avoid
- ❌ Don't duplicate content that exists in other docs
- ❌ Don't make the quick start more than 5 steps
- ❌ Don't use absolute paths for links
- ❌ Don't forget the WebGPU driver requirements
- ❌ Don't use raw cargo commands instead of just
- ❌ Don't create new documentation patterns
- ❌ Don't make it longer than necessary

## Success Confidence Score: 9/10

**High confidence** because:
- Extensive example documentation exists to follow
- Clear patterns established in examples/
- Architecture well documented in PLANNING.md
- All build commands documented in justfile
- No complex technical implementation required

**Minor risk**:
- Screenshot/GIF creation might require manual intervention