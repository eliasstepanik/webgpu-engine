# Execute BASE PRP

Implement a feature using using the PRP file.

## PRP File: $ARGUMENTS

## Execution Process

1. **Create Feature Branch**
   - Check current branch status with `git status`
   - Create a new feature branch: `git checkout -b feat/prp-<feature-name>`
   - Replace `<feature-name>` with a descriptive name from the PRP file

2. **Load PRP**
   - Read the specified PRP file
   - Understand all context and requirements
   - Follow all instructions in the PRP and extend the research if needed
   - Ensure you have all needed context to implement the PRP fully
   - Do more web searches and codebase exploration as needed

3. **ULTRATHINK**
   - Think hard before you execute the plan. Create a comprehensive plan addressing all requirements.
   - Break down complex tasks into smaller, manageable steps using your todos tools.
   - Use the TodoWrite tool to create and track your implementation plan.
   - Identify implementation patterns from existing code to follow.

4. **Execute the plan**
   - Execute the PRP
   - Implement all the code
   - **Commit changes incrementally** as you complete major milestones:
     ```bash
     git add <files>
     git commit -m "feat: <description of changes>"
     ```
   - Use proper commit message format (feat:, fix:, docs:, etc.)

5. **Test** (CRITICAL - YOU MUST TEST!)
   - **ALWAYS TEST YOUR IMPLEMENTATION!**
   - Use the SCENE environment variable for testing scene-related features:
     ```bash
     # Test specific scenes
     SCENE=test_mesh_generation cargo run
     SCENE=your_test_scene cargo run
     ```
   - Run each validation command
   - Test the feature manually to ensure it works as expected
   - Test edge cases and error conditions
   - Fix any failures and commit fixes
   - Re-run until all pass (You can test on windows using the windows-ssh-mcp)
   - **NO IMPLEMENTATION IS COMPLETE WITHOUT THOROUGH TESTING!**

6. **Complete**
   - If needed write follow-up requests in the .claude/requests/ folder with the .claude/requests/templates/INITIAL.md as a template.
   - If needed adjust the PLANNING.md to fit the current project.
   - Ensure all checklist items done
   - Run final validation suite
   - Report completion status
   - Read the PRP again to ensure you have implemented everything
   - Create a final git commit for any remaining changes
   - **Push the feature branch**: `git push -u origin feat/prp-<feature-name>`
   - Move PRP File to the `.claude/prp/done` folder
   - Report the branch name to the user for review/merge

7. **Reference the PRP**
   - You can always reference the PRP again if needed

Note: 
- All git commits must not mention Claude Code or Anthropic
- Use Author: Elias Stepanik, email: eliasstepanik@proton.me
- If validation fails, use error patterns in PRP to fix and retry
- The feature branch allows safe experimentation without affecting the main branch