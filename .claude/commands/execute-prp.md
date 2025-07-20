# Execute BASE PRP

Implement a feature using using the PRP file.

## PRP File: $ARGUMENTS

## Execution Process

1. **Load PRP**
   - Read the specified PRP file
   - Understand all context and requirements
   - Follow all instructions in the PRP and extend the research if needed
   - Ensure you have all needed context to implement the PRP fully
   - Do more web searches and codebase exploration as needed

2. **ULTRATHINK**
   - Think hard before you execute the plan. Create a comprehensive plan addressing all requirements.
   - Break down complex tasks into smaller, manageable steps using your todos tools.
   - Use the TodoWrite tool to create and track your implementation plan.
   - Identify implementation patterns from existing code to follow.

3. **Execute the plan**
   - Execute the PRP
   - Implement all the code

4. **Test** (CRITICAL - YOU MUST TEST!)
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
   - Fix any failures
   - Re-run until all pass (You can test on windows using the windows-ssh-mcp)
   - **NO IMPLEMENTATION IS COMPLETE WITHOUT THOROUGH TESTING!**

5. **Complete**
   - If needed write follow-up requests in the .claude/requests/ folder with the .claude/requests/templates/INITIAL.md as a template.
   - If needed adjust the PLANNING.md to fit the current project.
   - Ensure all checklist items done
   - Run final validation suite
   - Report completion status
   - Read the PRP again to ensure you have implemented everything
   - Create a branch with a fitting name.
   - Create a git commit. (Don´t mention Claude Code or anthropic in any of it, (Author: Elias Stepanik, email: eliasstepanik@proton.me))
   - Move PRP File to the `.claude/prp/done` folder


6. **Reference the PRP**
   - You can always reference the PRP again if needed

Note: If validation fails, use error patterns in PRP to fix and retry.