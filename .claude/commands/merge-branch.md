**Command Name:** `/merge-branch`

---

### Purpose

Safely merge two Git branches with automated conflict resolution strategies, comprehensive safety checks, and rollback capabilities to ensure bug-free operation.

---

### Usage

```bash
/merge-branch <source-branch> [into <target-branch>] [--strategy <strategy>]
```

*Examples:*

```bash
/merge-branch feature/new-ui                    # Merge into current branch
/merge-branch feature/new-ui into main          # Merge into specific branch
/merge-branch hotfix/bug-123 --strategy ours   # Use specific merge strategy
```

---

### What the Command Must Do

1. **Pre-Merge Validation**

   * Verify Git repository exists
   * Check for uncommitted changes and offer to stash
   * Ensure both branches exist locally or can be fetched
   * Create backup tag: `backup-pre-merge-YYYYMMDD-HHMMSS`
   * Verify no merge is already in progress

2. **Branch Preparation**

   * Fetch latest changes: `git fetch --all --prune`
   * Switch to target branch (default: current branch)
   * Pull latest changes with rebase: `git pull --rebase origin <target-branch>`
   * Verify source branch is up-to-date

3. **Dry-Run Merge Test**

   * Execute: `git merge --no-commit --no-ff <source-branch>`
   * Analyze conflicts:
     * Count conflicted files
     * Categorize conflicts (text vs binary)
     * Check for deleted/modified conflicts
   * Generate conflict report
   * Abort dry-run: `git merge --abort`

4. **Conflict Resolution Strategy**

   Based on analysis, apply resolution strategy:
   
   * **auto-safe** (default): Only auto-resolve trivial conflicts
     * Whitespace conflicts: `-Xignore-space-change`
     * Same-content conflicts: Auto-resolve
     * Complex conflicts: Prompt user
   
   * **ours**: Favor target branch changes (`-Xours`)
   * **theirs**: Favor source branch changes (`-Xtheirs`)
   * **manual**: Stop at conflicts for user resolution
   * **recursive**: Default Git recursive strategy
   * **patience**: Better for heavily refactored code (`-Xpatience`)

5. **Execute Merge**

   * Run merge with selected strategy
   * For each conflict:
     * Binary files: Prompt which version to keep
     * Text files: Apply resolution strategy
     * Deleted/modified: Intelligent resolution based on context
   * Track all resolutions in `.merge-log-YYYYMMDD-HHMMSS`

6. **Post-Merge Validation**

   * Run basic sanity checks:
     * No merge markers remain (`<<<<<<`, `======`, `>>>>>>`)
     * Build/compile test if applicable
     * Git status is clean
   * Generate merge summary report
   * Create post-merge tag: `merged-<source>-into-<target>-YYYYMMDD-HHMMSS`

7. **Rollback Capability**

   * If merge fails or user cancels:
     * `git merge --abort` if in progress
     * `git reset --hard backup-pre-merge-*` to restore
     * Clean up working directory
   * Log all actions for debugging

---

### Safety Features

1. **Automatic Backups**
   * Pre-merge backup tag
   * Stashed changes preservation
   * Merge log for audit trail

2. **Conflict Detection**
   * Dry-run before actual merge
   * Categorized conflict report
   * Binary file special handling

3. **Validation Steps**
   * Repository state verification
   * Branch existence checks
   * Clean working directory enforcement
   * Post-merge sanity checks

4. **User Prompts**
   * Stash uncommitted changes?
   * Proceed after conflict analysis?
   * Binary file resolution choice
   * Abort option at each step

---

### Expected Output

```
🔍 Analyzing merge: feature/user-auth → main

✓ Repository validated
✓ Branches fetched and updated
✓ Backup created: backup-pre-merge-20250114-143022

🧪 Dry-run analysis:
  - 3 files with conflicts
  - 2 text conflicts (auto-resolvable)
  - 1 binary conflict (requires choice)

📋 Conflict details:
  - src/auth.rs: Whitespace differences (auto-resolvable)
  - src/config.json: Merge conflict (auto-resolvable with 'ours' strategy)
  - assets/logo.png: Binary conflict (manual choice required)

Proceed with merge? [y/N]: y

🔧 Executing merge with 'auto-safe' strategy...
  ✓ src/auth.rs: Resolved (whitespace ignored)
  ✓ src/config.json: Resolved (ours strategy)
  ? assets/logo.png: Choose version:
    1) Keep current (main)
    2) Use incoming (feature/user-auth)
    Choice [1/2]: 1
  ✓ assets/logo.png: Resolved (kept current)

✅ Merge completed successfully!
  - 47 files changed
  - 3 conflicts resolved
  - Log saved: .merge-log-20250114-143045

Tagged as: merged-feature-user-auth-into-main-20250114-143045
```

---

### Error Handling

1. **Common Errors**
   * Not a Git repository → Clear error message
   * Dirty working directory → Offer to stash or abort
   * Branch doesn't exist → Attempt fetch, then error
   * Merge conflicts → Detailed conflict report
   * Failed validation → Automatic rollback

2. **Recovery Options**
   * `--abort`: Cancel at any point
   * `--rollback`: Restore to pre-merge state
   * `--continue`: Resume after manual fixes
   * `--skip`: Skip current file (dangerous)

---

### Quality Gate Checklist

* [ ] All Git commands use proper error handling
* [ ] Backup mechanisms tested and working
* [ ] Conflict resolution strategies documented
* [ ] User prompts are clear and actionable
* [ ] Rollback tested for all failure scenarios
* [ ] Output is informative but not overwhelming

---

### Implementation Notes

* Use `git` command with proper error code checking
* Parse Git output carefully (format can vary)
* Handle edge cases: empty commits, submodules, large files
* Support both local and remote branch references
* Maintain detailed log for debugging merge issues
* Consider `.gitattributes` merge strategies
* Respect existing Git hooks