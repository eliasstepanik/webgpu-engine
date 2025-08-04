**Command Name:** `/rebuild-claude-files`

---

### Purpose

Automatically rebuild all Claude-related files (CLAUDE.md, PLANNING.md, README.md, justfile) to match the current project structure, detecting project type and preserving user customizations.

---

### Usage

```bash
/rebuild-claude-files
```

*No arguments required - the command auto-detects project type.*

---

### What the Command Must Do

1. **Create backup directory**
   * Create `.claude/backups/` if not exists
   * Backup existing files with timestamp: `filename.YYYYMMDD-HHMMSS.backup`
   * Keep only last 5 backups per file

2. **Detect project type**
   * Run `cargo metadata --format-version 1 --no-deps` for Rust projects
   * Check dependencies in Cargo.toml/package.json/pyproject.toml
   * Detect:
     * `discord_bot` - Has serenity/twilight/discord.js deps
     * `game_engine` - Has wgpu/bevy/graphics deps
     * `web_app` - Has actix/rocket/axum/express deps
     * `cli_tool` - Binary with clap/structopt
     * `library` - Has [lib] section, no binary
     * `generic` - Default fallback

3. **Extract and preserve user settings**
   * Git author name and email from CLAUDE.md
   * Custom slash commands in `.claude/commands/`
   * Local settings in `.claude/settings.local.json`
   * Any sections marked with `#memory` tag
   * Project-specific rules and conventions

4. **Generate CLAUDE.md**
   * Load template from `.claude/templates/CLAUDE.md.<project_type>`
   * Fall back to `.claude/templates/CLAUDE.md.generic` if not found
   * Inject preserved user settings
   * Update project-specific sections
   * Ensure all paths match actual structure

5. **Generate PLANNING.md**
   * Use project type template
   * Generate actual directory tree with `find` or `tree`
   * Update architecture section based on project structure
   * List actual dependencies from lockfiles
   * Update build commands based on project type

6. **Generate README.md**
   * Project name from Cargo.toml/package.json
   * Installation instructions for project type
   * Usage examples appropriate to project
   * Development setup instructions
   * Contributing guidelines

7. **Update justfile**
   * Remove invalid package references
   * Add project-appropriate commands
   * Ensure all commands are valid
   * Add utility commands for the project type

8. **Create templates if missing**
   * Create `.claude/templates/` directory
   * Generate default templates for each project type
   * Mark TODO sections for customization

9. **Validate generated files**
   * Ensure no wrong project references (e.g., game engine in Discord bot)
   * Check all file paths are correct
   * Verify justfile commands syntax
   * Report any issues found

10. **Output summary**
    * List all files updated
    * Show detected project type
    * Note preserved settings
    * Display backup locations

---

### Expected Behavior

**Input state:**
```
Project with CLAUDE.md containing game engine instructions
but Cargo.toml shows Discord bot dependencies
```

**Output state:**
```
✓ Detected project type: discord_bot
✓ Backed up 4 files to .claude/backups/
✓ Preserved user settings:
  - Git author: Elias Stepanik <eliasstepanik@proton.me>
  - Custom commands: 3 found
  - Memory sections: 1 found
✓ Updated files:
  - CLAUDE.md (Discord bot template applied)
  - PLANNING.md (Updated structure and deps)
  - README.md (Discord bot setup instructions)
  - justfile (Discord-specific commands)
✓ Templates created in .claude/templates/
```

---

### Quality Gate Checklist

* [ ] Correctly detects project type from dependencies
* [ ] Preserves all user customizations
* [ ] Creates timestamped backups before overwriting
* [ ] Generated files match actual project structure
* [ ] No references to wrong project types
* [ ] All justfile commands are valid
* [ ] Templates created for future use

---

### Implementation Details

**Project Detection Priority:**
1. Check Cargo.toml dependencies first
2. Then package.json if present
3. Then pyproject.toml if present
4. Default to generic if unclear

**Template Naming Convention:**
```
.claude/templates/
├── CLAUDE.md.discord-bot
├── CLAUDE.md.game-engine
├── CLAUDE.md.generic
├── PLANNING.md.discord-bot
├── PLANNING.md.game-engine
└── PLANNING.md.generic
```

**Backup Cleanup:**
```bash
# Keep only 5 most recent backups per file
ls -t .claude/backups/CLAUDE.md.*.backup | tail -n +6 | xargs rm -f
```

**User Settings Regex Patterns:**
```regex
# Git author
Git author: (.+?) <(.+?)>

# Memory sections
#memory\n(.+?)(?=\n#|$)
```

---

### Error Handling

* If `cargo metadata` fails, try alternative detection methods
* If no templates exist, create minimal ones on the fly
* If backup fails, abort before overwriting
* Report all errors clearly with suggested fixes

---

### Notes

* This command is idempotent - running multiple times is safe
* Always creates backups before modifying files
* Templates can be customized after creation
* Respects .gitignore patterns when scanning project

---

## Implementation

When this command is executed, perform these steps:

### Step 1: Create Backup Directory
```bash
mkdir -p .claude/backups
```

### Step 2: Detect Project Type
```bash
# Check if Rust project
if [ -f "Cargo.toml" ]; then
    # Run cargo metadata
    cargo metadata --format-version 1 --no-deps > /tmp/cargo_metadata.json
    
    # Parse dependencies to detect type
    # Look for: wgpu/bevy → game_engine
    #           serenity/twilight → discord_bot
    #           actix/rocket/axum → web_app
    #           etc.
fi
```

### Step 3: Extract User Settings
```bash
# Extract from current CLAUDE.md
GIT_AUTHOR=$(grep -oP 'Git author: \K[^<]+' CLAUDE.md | xargs)
GIT_EMAIL=$(grep -oP 'Git author: .* <\K[^>]+' CLAUDE.md)
GITHUB_URL=$(grep -oP 'GitHub: \K.*' CLAUDE.md)
```

### Step 4: Backup Existing Files
```bash
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
for file in CLAUDE.md PLANNING.md README.md justfile; do
    if [ -f "$file" ]; then
        cp "$file" ".claude/backups/${file}.${TIMESTAMP}.backup"
    fi
done
```

### Step 5: Generate New Files
```bash
# Use detected project type
TEMPLATE_TYPE="${PROJECT_TYPE}"

# Copy template and replace placeholders
cp ".claude/templates/CLAUDE.md.${TEMPLATE_TYPE}" CLAUDE.md
sed -i "s/{{GIT_AUTHOR}}/${GIT_AUTHOR}/g" CLAUDE.md
sed -i "s/{{GIT_EMAIL}}/${GIT_EMAIL}/g" CLAUDE.md
sed -i "s/{{GITHUB_URL}}/${GITHUB_URL}/g" CLAUDE.md

# Generate project tree
find . -type f -name "*.rs" -o -name "*.toml" | head -50 > /tmp/project_tree.txt

# Similar for PLANNING.md
```

### Step 6: Cleanup Old Backups
```bash
# Keep only 5 most recent backups per file
for base_file in CLAUDE.md PLANNING.md README.md justfile; do
    ls -t .claude/backups/${base_file}.*.backup 2>/dev/null | tail -n +6 | xargs -r rm
done
```

### Step 7: Validate and Report
```bash
# Check files were created
echo "✓ Detected project type: ${PROJECT_TYPE}"
echo "✓ Backed up 4 files to .claude/backups/"
echo "✓ Preserved user settings:"
echo "  - Git author: ${GIT_AUTHOR} <${GIT_EMAIL}>"
echo "✓ Updated files:"
echo "  - CLAUDE.md"
echo "  - PLANNING.md"
echo "  - README.md"
echo "  - justfile"
```