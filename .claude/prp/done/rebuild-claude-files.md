name: "Rebuild Claude Files Command - Auto-generate project documentation"
description: |

## Purpose
Create a slash command that automatically rebuilds all Claude-related files (CLAUDE.md, PLANNING.md, README.md, justfile) to match the current project structure, detecting project type and preserving user customizations.

## Core Principles
1. **Auto-detection**: Detect project type without user input
2. **Preservation**: Keep user-specific settings and customizations  
3. **Safety**: Back up existing files before overwriting
4. **Accuracy**: Generate files that match actual project structure
5. **Global rules**: Follow all rules in CLAUDE.md

---

## Goal
Build a `/rebuild-claude-files` command that analyzes the current project and regenerates all Claude-related documentation files with appropriate content based on detected project type (Discord bot, game engine, web app, etc.).

## Why
- Projects often start with wrong templates or evolve over time
- Documentation drifts from reality as projects change
- Manual updates are error-prone and time-consuming
- New team members need accurate project documentation

## What
A command that:
- Detects project type from Cargo.toml, package.json, etc.
- Preserves user settings (git author, custom commands)
- Backs up existing files with timestamps
- Generates appropriate documentation for the detected project type
- Updates justfile/Makefile with correct commands

### Success Criteria
- [ ] Correctly detects Discord bot vs game engine vs other project types
- [ ] Preserves user-specific settings from existing files
- [ ] Creates backups before overwriting
- [ ] Generated files match actual project structure
- [ ] Justfile commands work with the actual project

## All Needed Context

### Documentation & References
```yaml
# MUST READ - Include these in your context window
- url: https://doc.rust-lang.org/cargo/commands/cargo-metadata.html
  why: Official docs for detecting Rust project structure
  
- url: https://docs.rs/cargo_metadata/latest/cargo_metadata/
  why: Rust crate for parsing cargo metadata programmatically
  
- file: .claude/commands/check-docs.md
  why: Pattern for analyzing and updating documentation files
  
- file: .claude/commands/generate-request.md
  why: Pattern for creating new files with templates

- url: https://apidog.com/blog/claude-md/
  why: Best practices for CLAUDE.md file structure
  
- url: https://www.anthropic.com/engineering/claude-code-best-practices
  why: Official Claude Code best practices

- file: .claude/prp/templates/prp_base.md
  why: Shows template structure and patterns
```

### Current Codebase tree
```bash
.
├── .claude/
│   ├── commands/           # Slash commands
│   │   ├── check-docs.md
│   │   ├── execute-prp.md
│   │   ├── generate-prp.md
│   │   └── generate-request.md
│   ├── prp/               # Project Review Protocols
│   ├── requests/          # Feature requests
│   └── settings.local.json
├── src/
│   └── main.rs           # Simple hello world
├── Cargo.toml            # Single crate project
├── CLAUDE.md             # Current AI guidelines (game engine content)
├── PLANNING.md           # Current planning (updated for Discord bot)
├── README.md             # Current readme (game engine content)
└── justfile              # Build commands (updated for Discord bot)
```

### Desired Codebase tree with files to be added
```bash
.
├── .claude/
│   ├── commands/
│   │   └── rebuild-claude-files.md  # NEW: This command
│   ├── templates/                   # NEW: Templates directory
│   │   ├── CLAUDE.md.discord-bot   # NEW: Discord bot template
│   │   ├── CLAUDE.md.game-engine   # NEW: Game engine template
│   │   ├── CLAUDE.md.generic       # NEW: Generic template
│   │   ├── PLANNING.md.discord-bot # NEW: Discord bot planning
│   │   ├── PLANNING.md.game-engine # NEW: Game engine planning
│   │   └── PLANNING.md.generic     # NEW: Generic planning
│   └── backups/                     # NEW: Backup directory
└── (existing files...)
```

### Known Gotchas & Library Quirks
```bash
# CRITICAL: cargo metadata requires valid Cargo.toml
# Will fail if Cargo.toml is malformed

# GOTCHA: Git author info might be in global config
# Check both .git/config and ~/.gitconfig

# PATTERN: Use timestamp suffix for backups
# Format: filename.YYYYMMDD-HHMMSS.backup

# CRITICAL: Preserve these user settings:
# - Git author name and email
# - Custom slash commands in .claude/commands/
# - Local settings in .claude/settings.local.json
# - Any #memory tagged content in CLAUDE.md
```

## Implementation Blueprint

### Data models and structure
```yaml
ProjectType:
  - discord_bot: Has discord/serenity/twilight deps
  - game_engine: Has wgpu/bevy/graphics deps  
  - web_app: Has actix/rocket/axum deps
  - cli_tool: Binary with clap/structopt
  - library: Has [lib] section, no binary
  - generic: Default fallback

ProjectStructure:
  - is_workspace: bool (multiple members)
  - crate_names: Vec<String>
  - main_language: rust/python/js/etc
  - build_system: just/make/cargo/npm
```

### List of tasks to be completed

```yaml
Task 1 - Create the command file:
CREATE .claude/commands/rebuild-claude-files.md:
  - Use standard command structure from other commands
  - Include clear usage instructions
  - Add self-validation steps

Task 2 - Detect project structure:
IMPLEMENT project detection logic:
  - Run `cargo metadata --format-version 1 --no-deps`
  - Parse JSON output to detect workspace vs single crate
  - Check Cargo.toml dependencies for project type hints
  - Check for package.json, pyproject.toml for non-Rust projects
  - Default to "generic" if unclear

Task 3 - Extract user settings to preserve:
SCAN existing files for user content:
  - Git author from CLAUDE.md commit section
  - Custom commands from .claude/commands/
  - Local settings from .claude/settings.local.json
  - Any sections marked with #memory tag
  - Project-specific rules and conventions

Task 4 - Create backup system:
CREATE backup directory and files:
  - Create .claude/backups/ if not exists
  - Copy existing files with timestamp suffix
  - Log which files were backed up
  - Clean up old backups (keep last 5)

Task 5 - Generate new CLAUDE.md:
CREATE new CLAUDE.md based on project type:
  - Start with appropriate template
  - Inject preserved user settings
  - Update project-specific sections
  - Ensure all paths match actual structure
  - Update dependency list from Cargo.toml/package.json

Task 6 - Generate new PLANNING.md:
CREATE new PLANNING.md:
  - Use project type template
  - Generate actual directory tree with find/tree
  - Update architecture section
  - List actual dependencies
  - Update build commands

Task 7 - Generate new README.md:
CREATE appropriate README.md:
  - Project name and description
  - Installation instructions
  - Usage examples for the project type
  - Development setup
  - Contributing guidelines

Task 8 - Update justfile:
UPDATE justfile with correct commands:
  - Remove invalid package references
  - Add project-appropriate commands
  - Ensure all commands work
  - Add new utility commands

Task 9 - Create templates:
CREATE template files in .claude/templates/:
  - Extract common patterns from existing files
  - Create variants for each project type
  - Include TODO markers for customization
  - Make templates minimal but complete

Task 10 - Final validation:
VALIDATE all generated files:
  - Ensure no game engine references in Discord bot
  - Check all file paths are correct
  - Verify justfile commands work
  - Ensure templates were used correctly
```

### Per task pseudocode

```python
# Task 2 - Project detection
def detect_project_type():
    # Check for Rust project
    if os.path.exists("Cargo.toml"):
        # Run cargo metadata
        result = subprocess.run(["cargo", "metadata", "--format-version", "1", "--no-deps"], 
                               capture_output=True, text=True)
        metadata = json.loads(result.stdout)
        
        # Check workspace
        is_workspace = len(metadata["workspace_members"]) > 1
        
        # Read Cargo.toml for dependencies
        with open("Cargo.toml") as f:
            cargo_content = f.read()
            
        # Detect type from dependencies
        if "serenity" in cargo_content or "twilight" in cargo_content:
            return "discord_bot", is_workspace
        elif "wgpu" in cargo_content or "bevy" in cargo_content:
            return "game_engine", is_workspace
        elif "actix" in cargo_content or "rocket" in cargo_content:
            return "web_app", is_workspace
        else:
            return "rust_generic", is_workspace
    
    # Check for other project types
    elif os.path.exists("package.json"):
        return "node_project", False
    elif os.path.exists("pyproject.toml"):
        return "python_project", False
    else:
        return "generic", False

# Task 3 - Extract user settings
def extract_user_settings():
    settings = {
        "git_author": None,
        "git_email": None,
        "custom_rules": [],
        "memory_sections": []
    }
    
    # Extract from CLAUDE.md
    if os.path.exists("CLAUDE.md"):
        with open("CLAUDE.md") as f:
            content = f.read()
            # Find git author line
            match = re.search(r"Git author: (.+?) <(.+?)>", content)
            if match:
                settings["git_author"] = match.group(1)
                settings["git_email"] = match.group(2)
            
            # Find #memory tagged sections
            memory_sections = re.findall(r"#memory\n(.+?)(?=\n#|$)", content, re.DOTALL)
            settings["memory_sections"] = memory_sections
    
    return settings

# Task 5 - Generate CLAUDE.md
def generate_claude_md(project_type, is_workspace, user_settings):
    # Load template
    template_path = f".claude/templates/CLAUDE.md.{project_type}"
    if not os.path.exists(template_path):
        template_path = ".claude/templates/CLAUDE.md.generic"
    
    with open(template_path) as f:
        template = f.read()
    
    # Replace placeholders
    content = template
    content = content.replace("{{PROJECT_TYPE}}", project_type)
    content = content.replace("{{IS_WORKSPACE}}", str(is_workspace))
    
    # Inject user settings
    if user_settings["git_author"]:
        content = re.sub(
            r"Git author: .+",
            f"Git author: {user_settings['git_author']} <{user_settings['git_email']}>",
            content
        )
    
    # Add memory sections
    for section in user_settings["memory_sections"]:
        content += f"\n\n#memory\n{section}"
    
    return content
```

### Integration Points
```yaml
FILESYSTEM:
  - create: .claude/templates/ directory
  - create: .claude/backups/ directory
  - read: Cargo.toml, package.json, etc.
  - write: CLAUDE.md, PLANNING.md, README.md, justfile
  
COMMANDS:
  - use: cargo metadata for Rust projects
  - use: find/tree for directory structure
  - use: git config for author info
  
PATTERNS:
  - follow: Existing command structure in .claude/commands/
  - preserve: User customizations and settings
  - backup: Always backup before overwriting
```

## Validation Loop

### Level 1: Command Creation
```bash
# Verify command file is valid markdown
cat .claude/commands/rebuild-claude-files.md

# Expected: Well-formed markdown with clear instructions
```

### Level 2: Project Detection
```bash
# Test detection on current project
cargo metadata --format-version 1 --no-deps | jq '.workspace_members | length'

# Expected: 1 for single crate, >1 for workspace
```

### Level 3: Backup System
```bash
# Verify backups are created
ls -la .claude/backups/

# Expected: Timestamped backup files
```

### Level 4: Generated Files
```bash
# Check generated files are valid
just preflight  # Should work with new justfile
grep "discord_bot" CLAUDE.md  # Should match project type
find . -name "*.md" -exec grep -l "game engine" {} \;  # Should be empty for Discord bot
```

### Level 5: Template System
```bash
# Verify templates exist and are valid
ls -la .claude/templates/
wc -l .claude/templates/*.md  # Each should have content

# Expected: Multiple template files with appropriate content
```

## Final Validation Checklist
- [ ] Command works: `/rebuild-claude-files`
- [ ] Correctly detects project type
- [ ] Preserves user settings (git author, etc.)
- [ ] Creates backups before overwriting
- [ ] Generated CLAUDE.md matches project
- [ ] Generated PLANNING.md has correct structure
- [ ] Generated README.md is appropriate
- [ ] Justfile commands all work
- [ ] No references to wrong project type
- [ ] Templates created for future use

---

## Anti-Patterns to Avoid
- ❌ Don't lose user customizations
- ❌ Don't overwrite without backing up
- ❌ Don't mix project types (game engine in Discord bot)
- ❌ Don't hardcode paths that might not exist
- ❌ Don't assume Rust-only projects
- ❌ Don't create commands that don't work

## Confidence Score: 8/10

The PRP is comprehensive with clear detection logic, preservation of user settings, and validation steps. The implementation is straightforward with good patterns to follow from existing commands. Minor complexity in template management and multi-language support prevents a perfect score.