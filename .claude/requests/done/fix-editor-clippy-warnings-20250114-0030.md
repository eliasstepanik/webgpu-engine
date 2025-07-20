## FEATURE:
Fix clippy warnings in editor hierarchy drag-drop implementation

## EXAMPLES:
editor/src/panels/hierarchy.rs – drag-drop implementation with format string and collapsible if warnings

## DOCUMENTATION:
https://rust-lang.github.io/rust-clippy/master/index.html#uninlined_format_args
https://rust-lang.github.io/rust-clippy/master/index.html#collapsible_if

## OTHER CONSIDERATIONS:
- Two format string warnings at lines 287 and 384 need inline variables
- Two collapsible if statement warnings at lines 323 and 420
- All warnings are in the hierarchy panel drag-drop implementation
- Fix should maintain existing functionality