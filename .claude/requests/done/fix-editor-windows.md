## FEATURE:

Fix the editor functionality on Windows systems. The editor component needs to be properly configured and tested to work seamlessly on Windows environments, including proper path handling, window management, and platform-specific features.

## EXAMPLES:

- The editor should be tested using the Windows SSH MCP connection to ensure proper functionality
- Examples in the `.claude/examples/` folder should demonstrate editor integration and work correctly on Windows
- The `scene_demo.rs` example should showcase editor capabilities when running on Windows

## DOCUMENTATION:

- Windows-specific WebGPU documentation: https://github.com/gfx-rs/wgpu
- Platform-specific path handling in Rust standard library documentation
- Windows SSH MCP server documentation for testing connectivity
- Any existing editor documentation in the `.claude/documentation/` folder

## OTHER CONSIDERATIONS:

- Use `mcp__windows-ssh-mcp__exec` for testing on the Windows machine
- Ensure proper path handling (forward vs backslash) for Windows file systems
- Check for any platform-specific window management issues with winit/WebGPU
- Verify that hot-reload functionality works correctly on Windows
- Test file watching and change detection on Windows NTFS file systems
- Ensure proper handling of Windows-specific permissions and file locks
- Consider Windows Defender or antivirus software that might interfere with file operations
- Test with both PowerShell and Command Prompt environments