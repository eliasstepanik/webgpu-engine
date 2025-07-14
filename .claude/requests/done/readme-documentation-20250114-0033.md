## FEATURE:
Create a comprehensive README that documents the WebGPU engine's architecture, features, setup instructions, and usage patterns following Rust game engine best practices

## EXAMPLES:
examples/README.md – Well-structured multi-viewport editor documentation with clear overview and quick start
examples/TUTORIAL.md – Comprehensive step-by-step tutorial format with troubleshooting sections
examples/QUICK_REFERENCE.md – Concise reference guide with tables for shortcuts and workflows
game/assets/scenes/README.md – Demo scene documentation showing JSON structure and API usage
.claude/documentation/viewport-debugging-guide.md – Technical guide with problem/solution format

## DOCUMENTATION:
https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html
https://doc.rust-lang.org/cargo/reference/manifest.html#the-readme-field
https://users.rust-lang.org/t/best-practice-for-documenting-crates-readme-md-vs-documentation-comments/124254
https://www.rapidinnovation.io/post/rust-game-engines-the-complete-guide-for-modern-game-development
https://rodneylab.com/rust-for-gaming/

## OTHER CONSIDERATIONS:
- README should integrate with Cargo.toml readme field for crates.io publishing
- Include badges for build status, crates.io version, and documentation
- Follow Rust documentation convention: brief intro → detailed explanation → code examples
- Highlight WebGPU requirements and platform support clearly
- Include both quick start (just run) and detailed setup instructions
- Reference existing layout examples in examples/ directory
- Document the three-crate workspace structure (engine, editor, game)
- Add troubleshooting section for common WebGPU driver issues
- Include GIF/screenshots of editor in action if possible
- Link to PLANNING.md for architecture details
- Mention the scripting system with Rhai as a key differentiator
- Use ASCII diagrams for architecture overview like in existing docs