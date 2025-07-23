# Check Documentation Alignment

This command analyzes CLAUDE.md and PLANNING.md to verify they still match the current project structure and provide relevant guidance. It will automatically fix the directory structure in PLANNING.md if mismatches are found.

## Steps to perform:

1. **Analyze Project Structure**
   - Check if the project is a workspace or single package
   - List all crates/members if workspace
   - Identify key directories and their purposes
   - Check for any special build configurations
   - Generate the actual directory tree structure

2. **Review CLAUDE.md**
   - Verify if the agent guidelines match current project type
   - Check if mentioned tools/commands exist (e.g., `just preflight`)
   - Ensure file paths and module structures are accurate
   - Verify if the testing/logging frameworks mentioned are actually used

3. **Review PLANNING.md**
   - Check if directory layout matches actual structure
   - **Automatically update the directory structure section if it doesn't match reality**
   - Verify build commands are accurate
   - Ensure the project purpose is defined (not placeholder)
   - Confirm that future considerations align with current state

4. **Fix Directory Structure in PLANNING.md**
   - Generate actual directory tree using `tree` or `find` commands
   - Replace the directory structure section in PLANNING.md with the actual structure
   - Preserve the markdown formatting and section headers
   - Include relevant directories but exclude build artifacts (target/, .git/, etc.)

5. **Generate Report**
   - List any mismatches found
   - Indicate which fixes were applied automatically
   - Suggest specific updates needed for remaining issues
   - Highlight outdated or irrelevant sections
   - Recommend additions for missing information

## Key checks:
- Is this a workspace or single package?
- Do the build commands in docs match justfile?
- Are the mentioned dependencies actually in Cargo.toml?
- Do the directory structures in docs match reality?
- Are there new modules/crates not documented?
- Are the coding standards still relevant?

## Output Format:
Provide a clear summary with:
- ✅ What's still accurate
- 🔧 What was automatically fixed
- ❌ What still needs manual updating
- 💡 Suggestions for improvements

## Templates Available:
When suggesting updates, refer to the templates in `.claude/templates/`:
- `CLAUDE.md` - AI agent guidelines template
- `PLANNING.md` - Project planning template

These templates contain TODO sections that should be filled in based on the actual project structure and requirements.