# Window Manager Borrow Checker Fix

## Issue Fixed: ✅

### Problem:
The `resize_window` method had a borrow checker error where:
1. `self.windows.get_mut(&window_id)` created a mutable borrow
2. `self.validate_surface_config()` tried to borrow `self` immutably
3. This created overlapping borrows

### Solution:
Restructured the code to avoid overlapping borrows by:
1. First getting surface capabilities with an immutable borrow
2. Dropping that borrow
3. Then getting mutable access to update the window data
4. Inlining the validation logic instead of calling a separate method

### Code Changes:
- Removed the separate `validate_surface_config` method
- Inlined the validation logic directly in `resize_window`
- Split the borrows into two separate scopes

### Result:
- No more borrow checker errors
- Cleaner code without the unused helper method
- Same functionality maintained

The window manager can now properly handle window resizing without compilation errors!